//! Everything related to DICOM tag data extraction.
use dicom::core::value::{CastValueError, ConvertValueError};
use dicom::core::DataDictionary;
use dicom::dictionary_std::{tags, StandardDataDictionary};
use dicom::object::{DefaultDicomObject, Tag};
use std::borrow::Cow;
use std::cell::RefCell;

/// Value used if the element for a DICOM tag is not found.
///
/// I'm not sure what to put here, see
/// https://github.com/FNNDSC/pypx/wiki/How-pypx-handles-missing-elements
pub const NOT_DEFINED: &str = "Not defined";

/// DICOM tag data reader.
///
/// Reading of DICOM tag data is fallible. If any errors occurs while trying to read data,
/// some default value is returned instead, and the error is recorded in `errors`.
pub(crate) struct TagExtractor<'a> {
    pub dcm: &'a DefaultDicomObject,
    pub errors: RefCell<Vec<DicomTagAndError>>,
}

/// A DICOM tag and the error which occurred when trying to read its value.
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
/// Some elements are assumed to must exist, some are allowed to not be defined.
/// I'm just really, really hoping that UID and ID numbers exist!
#[allow(non_snake_case)]
pub(crate) struct CommonElements<'a> {
    // these are all part of the path name.
    pub InstanceNumber: Option<&'a str>,
    pub SOPInstanceUID: &'a str,
    pub PatientID: &'a str,
    pub PatientName: Option<&'a str>,
    pub PatientBirthDate: Option<&'a str>,
    pub StudyDescription: Option<&'a str>,
    pub AccessionNumber: Option<&'a str>,
    pub StudyDate: Option<&'a str>,

    pub SeriesNumber: Option<MaybeU32<'a>>,
    pub SeriesDescription: Option<&'a str>,

    // these are not part of the path name, but used in the log path names.
    pub StudyInstanceUID: String,
    pub SeriesInstanceUID: String,
}

/// Something that is maybe a [u32], but in case it's not valid, is a [str].
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub(crate) enum MaybeU32<'a> {
    U32(u32),
    Str(&'a str),
}

impl<'a> From<&'a str> for MaybeU32<'a> {
    fn from(value: &'a str) -> Self {
        value
            .parse()
            .map(Self::U32)
            .unwrap_or_else(|_| MaybeU32::Str(value))
    }
}

impl<'a> ToString for MaybeU32<'a> {
    fn to_string(&self) -> String {
        match self {
            MaybeU32::U32(num) => num.to_string(),
            MaybeU32::Str(s) => s.to_string(),
        }
    }
}

impl<'a> TagExtractor<'a> {
    pub fn new(dcm: &'a DefaultDicomObject) -> Self {
        Self {
            dcm,
            errors: RefCell::new(Vec::new()),
        }
    }

    /// Get the value of a tag as a str. In case of failure,
    /// record the error in `self.errors` and return `""`.
    pub fn get(&self, tag: Tag) -> Cow<str> {
        self.dcm
            .element(tag)
            .map_err(DicomTagError::from)
            .and_then(|ele| ele.to_str().map_err(DicomTagError::from))
            .unwrap_or_else(|error| {
                let e = DicomTagAndError { tag, error };
                self.errors.borrow_mut().push(e);
                NOT_DEFINED.into()
            })
    }

    // /// Get the value of a tag as an integer. In the case of a failure,
    // /// record the error in `self.errors` and return [i32::MIN].
    // /// That oughta throw a wrench in the system!
    // pub fn get_i32(&self, tag: Tag) -> i32 {
    //     self.dcm
    //         .element(tag)
    //         .map_err(DicomTagError::from)
    //         .and_then(|ele| ele.to_int::<i32>().map_err(DicomTagError::from))
    //         .unwrap_or_else(|error| {
    //             let e = DicomTagAndError { tag, error };
    //             self.errors.borrow_mut().push(e);
    //             i32::MIN
    //         })
    // }
}

impl<'a> TryFrom<&'a DefaultDicomObject> for CommonElements<'a> {
    type Error = DicomTagError;
    fn try_from(dcm: &'a DefaultDicomObject) -> Result<Self, Self::Error> {
        // NOTE: the implementation here is optimized based on implementation details of dicom-rs v0.5.4.
        // - dcm.element(...)?.string() produces a reference to the data w/o cloning nor parsing
        // - dcm.element is more efficient than dcm.element_by_name, since the latter does a map lookup
        let data = Self {
            InstanceNumber: tt(dcm, tags::INSTANCE_NUMBER).ok(),
            SOPInstanceUID: tt(dcm, tags::SOP_INSTANCE_UID)?,
            PatientID: tt(dcm, tags::PATIENT_ID)?,
            PatientName: tt(dcm, tags::PATIENT_NAME).ok(),
            PatientBirthDate: tt(dcm, tags::PATIENT_BIRTH_DATE).ok(),
            StudyDescription: tt(dcm, tags::STUDY_DESCRIPTION).ok(),
            AccessionNumber: tt(dcm, tags::ACCESSION_NUMBER).ok(),
            StudyDate: tt(dcm, tags::STUDY_DATE).ok(),
            SeriesNumber: tt(dcm, tags::SERIES_NUMBER).map(MaybeU32::from).ok(),
            SeriesDescription: tt(dcm, tags::SERIES_DESCRIPTION).ok(),
            StudyInstanceUID: tts(dcm, tags::STUDY_INSTANCE_UID)?,
            SeriesInstanceUID: tts(dcm, tags::SERIES_INSTANCE_UID)?,
        };
        Ok(data)
    }
}

/// Get the trimmed `&str` to a DICOM object.
///
/// I tried to make this helper function low-cost.
fn tt(dcm: &DefaultDicomObject, tag: Tag) -> Result<&str, DicomTagError> {
    dcm.element(tag)
        .map_err(DicomTagError::from)
        .and_then(|e| e.string().map(|s| s.trim()).map_err(DicomTagError::from))
}

fn tts(dcm: &DefaultDicomObject, tag: Tag) -> Result<String, DicomTagError> {
    tt(dcm, tag).map(|s| s.replace('\0', ""))
}

/// Get the standard name of a tag.
pub(crate) fn name_of(tag: Tag) -> Option<&'static str> {
    // WHY SAG-anon has a DICOM tag (0019,0010)?
    StandardDataDictionary.by_tag(tag).map(|e| e.alias)
}
