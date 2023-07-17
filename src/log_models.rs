//! Models of what gets written to `/home/dicom/log`.
#![allow(non_snake_case)]

use crate::dicom_info::DicomInfo;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PatientData<'a> {
    pub PatientID: Cow<'a, str>,
    pub PatientName: Cow<'a, str>,
    pub PatientAge: Cow<'a, str>,
    pub PatientSex: Cow<'a, char>,
    pub PatientBirthDate: Cow<'a, str>,
    pub StudyList: HashSet<String>,
}

impl<'a> From<&'a DicomInfo> for PatientData<'a> {
    fn from(d: &'a DicomInfo) -> Self {
        Self {
            PatientID: Cow::Borrowed(&d.PatientID),
            PatientName: Cow::Borrowed(&d.PatientName),
            PatientAge: Cow::Borrowed(&d.PatientAge),
            PatientSex: Cow::Borrowed(&d.PatientSex),
            PatientBirthDate: Cow::Borrowed(&d.PatientBirthDate),
            StudyList: HashSet::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct StudyDataMeta<'a> {
    PatientID: &'a str,
    StudyDescription: &'a str,
    StudyDate: &'a str,
    StudyInstanceUID: &'a str,
    PerformedStationAETitle: &'a str,
}

#[derive(Debug, Serialize)]
struct StudyDataSeriesMeta {
    SeriesInstanceUID: String,
    SeriesBaseDir: String,
    DICOM: HashMap<String, ValueAndLabel>,
}

#[derive(Debug, Serialize)]
struct ValueAndLabel {
    value: String,
    label: String,
}

#[derive(Debug, Serialize)]
struct SeriesDataMeta<'a> {
    PatientID: &'a str,
    StudyInstanceUID: &'a str,
    SeriesInstanceUID: &'a str,
    SeriesDescription: &'a str,
    SeriesNumber: u32,
    SeriesDate: &'a str,
    Modality: &'a str,
}

#[derive(Debug, Serialize)]
pub(crate) struct SeriesPack {
    pub seriesPack: bool,
}

pub(crate) const SERIES_PACK: SeriesPack = SeriesPack { seriesPack: true };

#[derive(Debug, Serialize)]
struct InstanceData<'a> {
    PatientID: &'a str,
    StudyInstanceUID: &'a str,
    SeriesInstanceUID: &'a str,
    SeriesDescription: &'a str,
    SeriesNumber: u32,
    SeriesDate: &'a str,
    Modality: &'a str,
    outputFile: &'a str,
    imageObj: HashMap<String, FileStat>,
}

/// Unimportant information about a file's stat metadata.
/// Not complete.
/// https://github.com/FNNDSC/pypx/blob/7619c15f4d2303d6d5ca7c255d81d06c7ab8682b/pypx/smdb.py#L1306-L1317
#[derive(Debug, Serialize)]
struct FileStat {
    FSlocation: String,
}
