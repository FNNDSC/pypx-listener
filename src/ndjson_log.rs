use crate::dicom_data::DicomTagError;
use camino::{Utf8Path, Utf8PathBuf};
use dicom::object::Tag;
use hashbrown::HashMap;
use serde::Serialize;
use std::os::unix::fs::MetadataExt;

/// Produce a JSON string which describes the outcome of `rx-repack`.
pub fn json_message(
    src: &Utf8Path,
    result: &anyhow::Result<(Utf8PathBuf, HashMap<Tag, DicomTagError>)>,
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
    err: Option<String>,
}

impl<'a> Message<'a> {
    fn new(
        src: &'a Utf8Path,
        result: &'a anyhow::Result<(Utf8PathBuf, HashMap<Tag, DicomTagError>)>,
    ) -> Self {
        result
            .as_ref()
            .map_err(|e| anyhow::Error::msg(e.to_string())) // FIXME
            .and_then(|(dst, _warnings)| {
                fs_err::metadata(dst)
                    .map(|m| (dst, m.size()))
                    .map_err(anyhow::Error::from)
            })
            .map(|(dst, size)| Self {
                src,
                dst: Some(dst),
                size: Some(size),
                err: None,
            })
            .unwrap_or_else(|e| Self {
                src,
                dst: None,
                size: None,
                err: Some(e.to_string()),
            })
    }
}
