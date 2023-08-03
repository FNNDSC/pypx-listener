//! Models of what gets written to `/home/dicom/log`.
#![allow(non_snake_case)]
use crate::dicom_data::{CommonElements, TagExtractor};
use dicom::dictionary_std::tags;
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

#[derive(Debug, Serialize)]
pub(crate) struct StudyDataMeta<'a> {
    PatientID: &'a str,
    StudyDescription: &'a str,
    StudyDate: &'a str,
    StudyInstanceUID: &'a str,
    PerformedStationAETitle: Cow<'a, str>,
}

impl<'a> StudyDataMeta<'a> {
    pub fn new(d: &'a TagExtractor, e: &'a CommonElements) -> Self {
        Self {
            PatientID: e.PatientID,
            StudyDescription: e.StudyDescription,
            StudyDate: e.StudyDate,
            StudyInstanceUID: &e.StudyInstanceUID,
            PerformedStationAETitle: d.get(tags::PERFORMED_STATION_AE_TITLE),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct SeriesDataMeta<'a> {
    PatientID: &'a str,
    StudyInstanceUID: &'a str,
    SeriesInstanceUID: &'a str,
    SeriesDescription: &'a str,
    SeriesNumber: i32,
    SeriesDate: &'a str,
    Modality: Cow<'a, str>,
}

impl<'a> SeriesDataMeta<'a> {
    pub fn new(d: &'a TagExtractor, e: &'a CommonElements) -> Self {
        Self {
            PatientID: e.PatientID,
            StudyInstanceUID: &e.StudyInstanceUID,
            SeriesInstanceUID: &e.SeriesInstanceUID,
            SeriesDescription: e.SeriesDescription,
            SeriesNumber: e.SeriesNumber,
            SeriesDate: e.StudyDate,
            Modality: d.get(tags::MODALITY),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct InstanceData<'a> {
    PatientID: &'a str,
    StudyInstanceUID: &'a str,
    SeriesInstanceUID: &'a str,
    SeriesDescription: Cow<'a, str>,
    SeriesNumber: u32,
    SeriesDate: Cow<'a, str>,
    Modality: Cow<'a, str>,
    outputFile: &'a str,
    // TODO we don't include imageObj because I don't think it's used anywhwere.
    // Trying to get this information is annoying.
    imageObj: HashMap<&'a str, FileStat<'a>>,
}

impl<'a> InstanceData<'a> {
    pub fn new(
        d: &'a TagExtractor,
        e: &'a CommonElements,
        outputFile: &'a str,
        FSlocation: &'a str,
    ) -> Self {
        let imageObj = [(outputFile, FileStat { FSlocation })]
            .into_iter()
            .collect();
        Self {
            PatientID: e.PatientID,
            StudyInstanceUID: &e.StudyInstanceUID,
            SeriesInstanceUID: &e.SeriesInstanceUID,
            SeriesDescription: d.get(tags::SERIES_DESCRIPTION),
            SeriesNumber: d.get(tags::SERIES_NUMBER).parse().unwrap(), // FIXME
            SeriesDate: d.get(tags::SERIES_DATE),
            Modality: d.get(tags::MODALITY),
            outputFile,
            imageObj,
        }
    }
}

/// File's stat metadata.
/// Not complete.
/// https://github.com/FNNDSC/pypx/blob/7619c15f4d2303d6d5ca7c255d81d06c7ab8682b/pypx/smdb.py#L1306-L1317
#[derive(Debug, Serialize)]
struct FileStat<'a> {
    /// Important! Checked by smdb.py to count how many files are packed so far.
    FSlocation: &'a str,
}

#[derive(Debug, Serialize)]
pub(crate) struct SeriesPack {
    pub seriesPack: bool,
}

pub(crate) const SERIES_PACK: SeriesPack = SeriesPack { seriesPack: true };
