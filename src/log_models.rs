//! Models of what gets written to `/home/dicom/log`.
#![allow(non_snake_case)]
use crate::dicom_data::{name_of, CommonElements, TagExtractor};
use crate::errors::ElementSerializationError;
use dicom::core::header::Header;
use dicom::core::value::{DataSetSequence, DicomValueType, Value, ValueType};
use dicom::core::PrimitiveValue;
use dicom::dictionary_std::tags;
use dicom::object::mem::InMemElement;
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
    pub fn new(d: &'a TagExtractor, e: &'a CommonElements) -> Self {
        let PatientName = d.get(tags::PATIENT_NAME);
        let PatientAge = d.get(tags::PATIENT_AGE);
        let PatientSex = d.get(tags::PATIENT_SEX);
        Self {
            PatientID: Cow::Borrowed(e.PatientID),
            PatientName,
            PatientAge,
            PatientSex,
            PatientBirthDate: Cow::Borrowed(e.PatientBirthDate),
            StudyList: HashSet::new(),
        }
    }
}
//
// #[derive(Debug, Serialize)]
// pub(crate) struct StudyDataMeta<'a> {
//     PatientID: &'a str,
//     StudyDescription: &'a str,
//     StudyDate: &'a str,
//     StudyInstanceUID: &'a str,
//     PerformedStationAETitle: &'a str,
// }
//
// impl<'a> StudyDataMeta<'a> {
//     pub fn new(
//         d: &'a DefaultDicomObject,
//         e: &'a CommonElements,
//     ) -> Result<Self, DicomTagReadError> {
//         let data = Self {
//             PatientID: e.PatientID,
//             StudyDescription: e.StudyDescription,
//             StudyDate: e.StudyDate,
//             StudyInstanceUID: &e.StudyInstanceUID,
//             PerformedStationAETitle: tt(d, tags::PERFORMED_STATION_AE_TITLE).unwrap_or(""),
//         };
//         Ok(data)
//     }
// }
//

// fn serialize_loint(ele: &InMemElement) -> Result<String, DicomElementSerializationError> {
//     if let Ok(v) = ele.to_multi_int::<i64>() {
//         let s = serialize_first_or_as_list(v)?;
//         Ok(s)
//     } else {
//         let s = serialize_lostr(ele)?;
//         eprintln!("WARNING: Could not serialize {} as list of int", &s);
//         Ok(s)
//     }
// }
//
// fn serialize_lonum(ele: &InMemElement) -> Result<String, DicomElementSerializationError> {
//     if let Ok(v) = ele.to_multi_float64() {
//         let s = serialize_first_or_as_list(v)?;
//         Ok(s)
//     } else {
//         let s = serialize_lostr(ele)?;
//         eprintln!("WARNING: Could not serialize {} as list of float", &s);
//         Ok(s)
//     }
// }
//
// fn serialize_lostr(ele: &InMemElement) -> Result<String, DicomElementSerializationError> {
//     let mut values = ele.to_multi_str()?.to_vec();
//     let value = if values.len() == 1 {
//         values.swap_remove(0)
//     } else {
//         serde_json::to_string(&values)?
//     };
//     Ok(value)
// }
//
// fn serialize_first_or_as_list<T: Serialize>(v: Vec<T>) -> Result<String, serde_json::Error> {
//     if v.len() == 1 {
//         serde_json::to_string(&v[0])
//     } else {
//         serde_json::to_string(&v)
//     }
// }
//

//
// #[derive(Debug, Serialize)]
// pub(crate) struct SeriesDataMeta<'a> {
//     PatientID: &'a str,
//     StudyInstanceUID: &'a str,
//     SeriesInstanceUID: &'a str,
//     SeriesDescription: &'a str,
//     SeriesNumber: u32,
//     SeriesDate: &'a str,
//     Modality: &'a str,
// }
//
// impl<'a> SeriesDataMeta<'a> {
//     pub fn new(
//         d: &'a DefaultDicomObject,
//         e: &'a CommonElements,
//     ) -> Result<Self, DicomTagReadError> {
//         let data = Self {
//             PatientID: e.PatientID,
//             StudyInstanceUID: &e.StudyInstanceUID,
//             SeriesInstanceUID: &e.SeriesInstanceUID,
//             SeriesDescription: e.SeriesDescription,
//             SeriesNumber: e.SeriesNumber,
//             SeriesDate: e.StudyDate,
//             Modality: tt(d, tags::MODALITY)?,
//         };
//         Ok(data)
//     }
// }
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
//     imageObj: HashMap<String, FileStat>,
// }
//
// impl<'a> InstanceData<'a> {
//     pub fn new(
//         d: &'a DefaultDicomObject,
//         e: &'a CommonElements,
//         outputFile: &'a str,
//         FSlocation: String,
//     ) -> Result<Self, DicomTagReadError> {
//         let imageObj = [(outputFile.to_string(), FileStat { FSlocation })]
//             .into_iter()
//             .collect();
//         let data = Self {
//             PatientID: e.PatientID,
//             StudyInstanceUID: tt(d, tags::STUDY_INSTANCE_UID)?,
//             SeriesInstanceUID: tt(d, tags::SERIES_INSTANCE_UID)?,
//             SeriesDescription: tt(d, tags::SERIES_DESCRIPTION)?,
//             SeriesNumber: tt(d, tags::SERIES_NUMBER)?.parse()?,
//             SeriesDate: tt(d, tags::SERIES_DATE)?,
//             Modality: tt(d, tags::MODALITY)?,
//             outputFile,
//             imageObj,
//         };
//         Ok(data)
//     }
// }
//
// /// File's stat metadata.
// /// Not complete.
// /// https://github.com/FNNDSC/pypx/blob/7619c15f4d2303d6d5ca7c255d81d06c7ab8682b/pypx/smdb.py#L1306-L1317
// #[derive(Debug, Serialize)]
// struct FileStat {
//     /// Important! Checked by smdb.py to count how many files are packed so far.
//     FSlocation: String,
// }
//
// #[derive(Debug, Serialize)]
// pub(crate) struct SeriesPack {
//     pub seriesPack: bool,
// }
//
// pub(crate) const SERIES_PACK: SeriesPack = SeriesPack { seriesPack: true };
