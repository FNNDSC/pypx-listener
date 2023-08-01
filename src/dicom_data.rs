use dicom::core::value::{CastValueError, ConvertValueError};
use dicom::core::DataDictionary;
use dicom::dictionary_std::{tags, StandardDataDictionary};
use dicom::object::{DefaultDicomObject, Tag};
use serde::Serialize;
use std::borrow::Cow;
use std::cell::RefCell;

/// DICOM tag data reader.
///
/// Reading of DICOM tag data is fallible. If any errors occurs while trying to read data,
/// some default value is returned instead, and the error is recorded in `errors`.
pub(crate) struct TagExtractor<'a> {
    pub dcm: &'a DefaultDicomObject,
    pub errors: RefCell<Vec<DicomTagAndError>>,
}

pub struct DicomTagAndError {
    pub tag: Tag,
    pub error: DicomTagError,
}

/// Error reading a DICOM tag's value.
#[derive(thiserror::Error, Debug)]
pub enum DicomTagError {
    #[error(transparent)]
    Access(#[from] dicom::object::AccessError),
    #[error(transparent)]
    CastValue(#[from] CastValueError),
    #[error(transparent)]
    ConvertValue(#[from] ConvertValueError),
}

/// DICOM elements which a [PypxPath] is comprised of.
///
/// These DICOM elements _must_ be present and valid for `pypx`.
#[allow(non_snake_case)]
pub(crate) struct CommonElements<'a> {
    // these are all part of the path name.
    pub InstanceNumber: &'a str,
    pub SOPInstanceUID: &'a str,
    pub PatientID: &'a str,
    pub PatientName: &'a str,
    pub PatientBirthDate: &'a str,
    pub StudyDescription: &'a str,
    pub AccessionNumnber: &'a str,
    pub StudyDate: &'a str,
    pub SeriesNumber: i32, // SeriesNumber is of the "Integer String" (IS) type
    pub SeriesDescription: &'a str,

    // these are not part of the path name, but used in the log path names.
    pub StudyInstanceUID: String,
    pub SeriesInstanceUID: String,
}

impl<'a> TagExtractor<'a> {
    pub fn new(dcm: &'a DefaultDicomObject) -> Self {
        Self {
            dcm,
            errors: RefCell::new(Vec::new()),
        }
    }

    pub fn get(&self, tag: Tag) -> Cow<str> {
        self.dcm
            .element(tag)
            .map_err(DicomTagError::from)
            .and_then(|ele| ele.to_str().map_err(DicomTagError::from))
            .unwrap_or_else(|error| {
                let e = DicomTagAndError { tag, error };
                self.errors.borrow_mut().push(e);
                "".into()
            })
    }
}

impl<'a> TryFrom<&'a DefaultDicomObject> for CommonElements<'a> {
    type Error = DicomTagError;
    fn try_from(dcm: &'a DefaultDicomObject) -> Result<Self, Self::Error> {
        // NOTE: the implementation here is optimized based on implementation details of dicom-rs v0.5.4.
        // - dcm.element(...)?.string() produces a reference to the data w/o cloning nor parsing
        // - dcm.element is more efficient than dcm.element_by_name, since the latter does a map lookup
        let data = Self {
            InstanceNumber: tt(dcm, tags::INSTANCE_NUMBER)?,
            SOPInstanceUID: tt(dcm, tags::SOP_INSTANCE_UID)?,
            PatientID: tt(dcm, tags::PATIENT_ID)?,
            PatientName: tt(dcm, tags::PATIENT_NAME)?,
            PatientBirthDate: tt(dcm, tags::PATIENT_BIRTH_DATE)?,
            StudyDescription: tt(dcm, tags::STUDY_DESCRIPTION)?,
            AccessionNumnber: tt(dcm, tags::ACCESSION_NUMBER)?,
            StudyDate: tt(dcm, tags::STUDY_DATE)?,
            SeriesNumber: dcm
                .element(tags::SERIES_NUMBER)
                .map_err(Self::Error::from)
                .and_then(|ele| ele.value().to_int::<i32>().map_err(Self::Error::from))?,
            SeriesDescription: tt(dcm, tags::SERIES_DESCRIPTION)?,
            StudyInstanceUID: tts(dcm, tags::STUDY_INSTANCE_UID)?,
            SeriesInstanceUID: tts(dcm, tags::SERIES_INSTANCE_UID)?,
        };
        Ok(data)
    }
}

/// Get the `&str` to a DICOM object.
///
/// I tried to make this helper function low-cost.
fn tt(dcm: &DefaultDicomObject, tag: Tag) -> Result<&str, DicomTagError> {
    // TODO make this a method, and maybe we should call replace('\0', "")
    dcm.element(tag)
        .map_err(DicomTagError::from)
        .and_then(|e| e.string().map(|s| s.trim()).map_err(DicomTagError::from))
}

fn tts(dcm: &DefaultDicomObject, tag: Tag) -> Result<String, DicomTagError> {
    tt(dcm, tag).map(|s| s.replace('\0', ""))
}

pub(crate) fn name_of(tag: Tag) -> Option<&'static str> {
    // WHY SAG-anon has a DICOM tag (0019,0010)?
    StandardDataDictionary.by_tag(tag).map(|e| e.alias)
}
