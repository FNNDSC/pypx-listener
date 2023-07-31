//! Models of what gets written to `/home/dicom/log`.
#![allow(non_snake_case)]

use crate::errors::{DicomElementSerializationError, DicomTagReadError};
use crate::helpers::tt;
use crate::pack_path::PypxPathElements;
use dicom::core::header::Header;
use dicom::core::value::{CastValueError, Value};
use dicom::core::{DataDictionary, Tag, VR};
use dicom::dictionary_std::{tags, StandardDataDictionary};
use dicom::object::mem::{InMemElement, InMemFragment};
use dicom::object::{DefaultDicomObject, InMemDicomObject};
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PatientData<'a> {
    pub PatientID: Cow<'a, str>,
    pub PatientName: Cow<'a, str>,
    pub PatientAge: Cow<'a, str>,
    pub PatientSex: Cow<'a, str>,
    pub PatientBirthDate: Cow<'a, str>,
    pub StudyList: HashSet<String>,
}

impl<'a> PatientData<'a> {
    pub fn new(
        d: &'a DefaultDicomObject,
        e: &'a PypxPathElements,
    ) -> Result<Self, DicomTagReadError> {
        {
            let name = tt(&d, tags::PATIENT_NAME)?;
            let age = tt(&d, tags::PATIENT_AGE)?;
            let sex = tt(&d, tags::PATIENT_SEX)?;
            let patient = Self {
                PatientID: Cow::Borrowed(e.PatientID),
                PatientName: Cow::Borrowed(name),
                PatientAge: Cow::Borrowed(age),
                PatientSex: Cow::Borrowed(sex),
                PatientBirthDate: Cow::Borrowed(e.PatientBirthDate),
                StudyList: HashSet::new(),
            };
            Ok(patient)
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct StudyDataMeta<'a> {
    PatientID: &'a str,
    StudyDescription: &'a str,
    StudyDate: &'a str,
    StudyInstanceUID: &'a str,
    PerformedStationAETitle: &'a str,
}

impl<'a> StudyDataMeta<'a> {
    pub fn new(d: &'a DefaultDicomObject, e: &'a PypxPathElements) -> Result<Self, DicomTagReadError> {
        let data = Self {
            PatientID: e.PatientID,
            StudyDescription: e.StudyDescription,
            StudyDate: e.StudyDate,
            StudyInstanceUID: e.StudyInstanceUID,
            PerformedStationAETitle: tt(&d, tags::PERFORMED_STATION_AE_TITLE).unwrap_or(""),
        };
        Ok(data)
    }
}




#[derive(Debug, Serialize)]
pub(crate) struct StudyDataSeriesMeta {
    SeriesInstanceUID: String,
    SeriesBaseDir: String,
    DICOM: HashMap<String, ValueAndLabel>,
}

impl StudyDataSeriesMeta {
    pub fn new(
        SeriesInstanceUID: String,
        SeriesBaseDir: String,
        dcm: &DefaultDicomObject,
    ) -> Result<StudyDataSeriesMeta, CastValueError> {
        let DICOM = dcm
            .iter()
            .map(ValueAndLabel::try_from)
            .filter_map(|r| r.ok())
            .map(|v| (v.label.to_string(), v))
            .collect::<HashMap<String, ValueAndLabel>>();
        Ok(Self {
            SeriesInstanceUID,
            SeriesBaseDir,
            DICOM,
        })
    }
}

impl TryFrom<&InMemElement> for ValueAndLabel {
    type Error = DicomElementSerializationError;
    fn try_from(ele: &InMemElement) -> Result<Self, Self::Error> {
        let tag = ele.tag();
        if tag == tags::PIXEL_DATA {
            return Err(DicomElementSerializationError::Excluded(tag));
        }
        let label = name_of(tag)
            .ok_or_else(|| DicomElementSerializationError::UnknownTagError(tag))?
            .to_string();
        if matches!(ele.value(), Value::PixelSequence { .. }) {
            return Err(DicomElementSerializationError::Excluded(tag));
        }
        //
        // if &label == "ReferencedImageSequence" {
        //     dbg!(ele.value().to_multi_str().unwrap());
        // }
        //

        /// TODO serialize numbers and Item such as ReferencedImageSequence
        let mut values = ele.to_multi_str()?.to_vec();
        let value = if values.len() == 1 {
            values.swap_remove(0)
        } else {
            serde_json::to_string(&values)?
        };
        Ok(Self { label, value })
    }
}

fn name_of(tag: Tag) -> Option<&'static str> {
    // WHY SAG-anon has a DICOM tag (0019,0010)?
    StandardDataDictionary.by_tag(tag).map(|e| e.alias)
}

#[derive(Debug, Serialize)]
struct ValueAndLabel {
    value: String,
    label: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SeriesDataMeta<'a> {
    PatientID: &'a str,
    StudyInstanceUID: &'a str,
    SeriesInstanceUID: &'a str,
    SeriesDescription: &'a str,
    SeriesNumber: u32,
    SeriesDate: &'a str,
    Modality: &'a str,
}

impl<'a> SeriesDataMeta<'a> {
    pub fn new(d: &'a DefaultDicomObject, e: &'a PypxPathElements) -> Result<Self, DicomTagReadError> {
        let data = Self {
            PatientID: e.PatientID,
            StudyInstanceUID: e.StudyInstanceUID,
            SeriesInstanceUID: e.SeriesInstanceUID,
            SeriesDescription: e.SeriesDescription,
            SeriesNumber: e.SeriesNumber.clone(),
            SeriesDate: e.StudyDate,
            Modality: tt(d, tags::MODALITY)?,
        };
        Ok(data)
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct InstanceData<'a> {
    PatientID: &'a str,
    StudyInstanceUID: &'a str,
    SeriesInstanceUID: &'a str,
    SeriesDescription: &'a str,
    SeriesNumber: u32,
    SeriesDate: &'a str,
    Modality: &'a str,
    outputFile: &'a str,
    // TODO we don't include imageObj because I don't think it's used anywhwere.
    // Trying to get this information is annoying.
    imageObj: HashMap<String, FileStat>,
}

impl<'a> InstanceData<'a> {
    pub fn new(d: &'a DefaultDicomObject, e: &'a PypxPathElements, outputFile: &'a str, FSlocation: String) -> Result<Self, DicomTagReadError> {
        let imageObj = [(outputFile.to_string(), FileStat { FSlocation })].into_iter().collect();
        let data = Self {
            PatientID: e.PatientID,
            StudyInstanceUID: tt(d, tags::STUDY_INSTANCE_UID)?,
            SeriesInstanceUID: tt(d, tags::SERIES_INSTANCE_UID)?,
            SeriesDescription: tt(d, tags::SERIES_DESCRIPTION)?,
            SeriesNumber: tt(d, tags::SERIES_NUMBER)?.parse()?,
            SeriesDate: tt(d, tags::SERIES_DATE)?,
            Modality: tt(d, tags::MODALITY)?,
            outputFile,
            imageObj
        };
        Ok(data)
    }
}



/// File's stat metadata.
/// Not complete.
/// https://github.com/FNNDSC/pypx/blob/7619c15f4d2303d6d5ca7c255d81d06c7ab8682b/pypx/smdb.py#L1306-L1317
#[derive(Debug, Serialize)]
struct FileStat {
    /// Important! Checked by smdb.py to count how many files are packed so far.
    FSlocation: String,
}


#[derive(Debug, Serialize)]
pub(crate) struct SeriesPack {
    pub seriesPack: bool,
}

pub(crate) const SERIES_PACK: SeriesPack = SeriesPack { seriesPack: true };
