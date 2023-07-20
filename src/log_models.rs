//! Models of what gets written to `/home/dicom/log`.
#![allow(non_snake_case)]

use crate::errors::DicomTagReadError;
use crate::pack_path::PypxPathElements;
use crate::tt;
use dicom::core::header::Header;
use dicom::core::value::CastValueError;
use dicom::core::{DataDictionary, Tag};
use dicom::dictionary_std::{tags, StandardDataDictionary};
use dicom::object::DefaultDicomObject;
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
            let name = tt!(&d, tags::PATIENT_NAME)?;
            let age = tt!(&d, tags::PATIENT_AGE)?;
            let sex = tt!(&d, tags::PATIENT_SEX)?;
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
//
// #[derive(Debug, Serialize)]
// struct StudyDataMeta<'a> {
//     PatientID: &'a str,
//     StudyDescription: &'a str,
//     StudyDate: &'a str,
//     StudyInstanceUID: &'a str,
//     PerformedStationAETitle: &'a str,
// }
//
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
        // TODO: some elements are deeply nested, such as ReferencedImageSequence
        // TODO: SAG-anon has a DICOM tag (0019,0010)
        let DICOM = dcm
            .iter()
            .map(|ele| {
                let label = name_of(ele.tag())?.to_string();
                let value = ele.value().to_str().ok()?.to_string();
                Some((label.to_string(), ValueAndLabel { value, label }))
            })
            .filter_map(|e| e)
            .collect::<HashMap<String, ValueAndLabel>>();
        Ok(Self {
            SeriesInstanceUID,
            SeriesBaseDir,
            DICOM,
        })
    }
}

fn name_of(tag: Tag) -> Option<&'static str> {
    StandardDataDictionary.by_tag(tag).map(|e| e.alias)
}

impl TryFrom<&DefaultDicomObject> for StudyDataSeriesMeta {
    type Error = DicomTagReadError;
    fn try_from(dcm: &DefaultDicomObject) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug, Serialize)]
struct ValueAndLabel {
    value: String,
    label: String,
}

// #[derive(Debug, Serialize)]
// struct SeriesDataMeta<'a> {
//     PatientID: &'a str,
//     StudyInstanceUID: &'a str,
//     SeriesInstanceUID: &'a str,
//     SeriesDescription: &'a str,
//     SeriesNumber: u32,
//     SeriesDate: &'a str,
//     Modality: &'a str,
// }
//
// #[derive(Debug, Serialize)]
// pub(crate) struct SeriesPack {
//     pub seriesPack: bool,
// }
//
// pub(crate) const SERIES_PACK: SeriesPack = SeriesPack { seriesPack: true };
//
// #[derive(Debug, Serialize)]
// pub(crate) struct InstanceData<'a> {
//     PatientID: &'a str,
//     StudyInstanceUID: &'a str,
//     SeriesInstanceUID: &'a str,
//     SeriesDescription: &'a str,
//     SeriesNumber: u32,
//     SeriesDate: &'a str,
//     Modality: &'a str,
//     outputFile: &'a str,
//     // TODO we don't include imageObj because I don't think it's used anywhwere.
//     // Trying to get this information is annoying.
//     // imageObj: HashMap<String, FileStat>,
// }
//
// // /// Unimportant information about a file's stat metadata.
// // /// Not complete.
// // /// https://github.com/FNNDSC/pypx/blob/7619c15f4d2303d6d5ca7c255d81d06c7ab8682b/pypx/smdb.py#L1306-L1317
// // #[derive(Debug, Serialize)]
// // struct FileStat {
// //     FSlocation: String,
// // }
//
// impl<'a> From<&'a DicomInfo> for InstanceData<'a> {
//     fn from(d: &'a DicomInfo) -> Self {
//         Self {
//             PatientID: &d.PatientID,
//             StudyInstanceUID: &d.StudyInstanceUID,
//             SeriesInstanceUID: &d.SeriesInstanceUID,
//             SeriesDescription: &d.SeriesDescription,
//             SeriesNumber: d.SeriesNumber.clone(),
//             SeriesDate: &d.SeriesDate,
//             Modality: &d.Modality,
//             outputFile: &d.pypx_fname,
//         }
//     }
// }
