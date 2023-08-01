use crate::dicom_data::{name_of, DicomTagAndError, DicomTagError};
use camino::{Utf8Path, Utf8PathBuf};
use dicom::object::Tag;
use hashbrown::HashMap;
use serde::Serialize;
use std::os::unix::fs::MetadataExt;

/// Produce a JSON string which describes the outcome of `rx-repack`.
pub fn json_message(
    src: &Utf8Path,
    result: &anyhow::Result<(Utf8PathBuf, Vec<DicomTagAndError>)>,
) -> anyhow::Result<String> {
    let msg = Message::new(src, result);
    serde_json::to_string(&msg).map_err(anyhow::Error::from)
}

#[derive(Serialize, Debug)]
struct Message<'a> {
    src: &'a Utf8Path,
    dst: Option<&'a Utf8Path>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
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
    fn new(
        src: &'a Utf8Path,
        result: &'a anyhow::Result<(Utf8PathBuf, Vec<DicomTagAndError>)>,
    ) -> Self {
        match result {
            Ok((dst, missing)) => {
                let (size, error) = fs_err::metadata(dst)
                    .map(|metadata| (Some(metadata.size()), None))
                    .unwrap_or_else(|error| (None, Some(error.to_string())));
                Self {
                    src,
                    dst: Some(dst),
                    size,
                    error,
                    missing: missing.iter().map(DicomTagNameAndError::from).collect(),
                }
            }
            Err(e) => Self {
                src,
                dst: None,
                size: None,
                error: Some(e.to_string()),
                missing: Vec::new(),
            },
        }
    }
}
