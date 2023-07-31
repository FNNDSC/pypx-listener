use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;
use std::os::unix::fs::MetadataExt;

/// Produce a JSON string which describes the outcome of `rx-repack`.
pub(crate) fn json_message(
    src: &Utf8Path,
    result: anyhow::Result<Utf8PathBuf>,
) -> anyhow::Result<String> {
    let msg = Message::new(src, result);
    serde_json::to_string(&msg).map_err(anyhow::Error::from)
}

#[derive(Serialize, Debug)]
struct Message<'a> {
    src: &'a Utf8Path,
    dst: Option<Utf8PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    err: Option<String>,
}

impl<'a> Message<'a> {
    fn new(src: &'a Utf8Path, result: anyhow::Result<Utf8PathBuf>) -> Self {
        result
            .and_then(|dst| {
                fs_err::metadata(&dst)
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
