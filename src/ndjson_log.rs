use crate::dicom_data::{name_of, DicomTagAndError};
use crate::repack::RepackOutcome;
use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;
use std::os::unix::fs::MetadataExt;

/// Produce a JSON string which describes the outcome of `rx-repack`.
pub fn json_message(
    src: &Utf8Path,
    result: &anyhow::Result<RepackOutcome>,
) -> anyhow::Result<String> {
    let msg = Message::new(src, result);
    serde_json::to_string(&msg).map_err(anyhow::Error::from)
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
struct Message<'a> {
    src: &'a Utf8Path,
    dst: Option<&'a Utf8Path>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    PatientID: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    SeriesInstanceUID: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    missing: Vec<DicomTagNameAndError>,
}

#[derive(Serialize, Debug)]
pub struct DicomTagNameAndError {
    tag: String,
    name: Option<&'static str>,
    error: String,
}

impl From<&DicomTagAndError> for DicomTagNameAndError {
    fn from(value: &DicomTagAndError) -> Self {
        Self {
            tag: value.tag.to_string(),
            name: name_of(value.tag),
            error: value.error.to_string(),
        }
    }
}

impl<'a> Message<'a> {
    fn new(src: &'a Utf8Path, result: &'a anyhow::Result<RepackOutcome>) -> Self {
        match result {
            Ok(outcome) => {
                let (size, error) = fs_err::metadata(&outcome.dst)
                    .map(|metadata| (Some(metadata.size()), None))
                    .unwrap_or_else(|error| (None, Some(error.to_string())));
                Self {
                    src,
                    dst: Some(&outcome.dst),
                    size,
                    error,
                    missing: outcome
                        .missing
                        .iter()
                        .map(DicomTagNameAndError::from)
                        .collect(),
                    PatientID: Some(&outcome.PatientID),
                    SeriesInstanceUID: Some(&outcome.SeriesInstanceUID),
                }
            }
            Err(e) => Self {
                src,
                dst: None,
                size: None,
                error: Some(e.to_string()),
                missing: Vec::new(),
                PatientID: None,
                SeriesInstanceUID: None,
            },
        }
    }
}
