use crate::errors::DicomTagReadError;
use dicom::core::Tag;
use dicom::object::DefaultDicomObject;
use regex::Regex;
use std::sync::OnceLock;

/// Get the `&str` to a DICOM object.
///
/// I tried to make this helper function low-cost.
pub(crate) fn tt(dcm: &DefaultDicomObject, tag: Tag) -> Result<&str, DicomTagReadError> {
    // TODO make this a method, and maybe we should call replace('\0', "")
    dcm.element(tag).map_err(|e| e.into()).and_then(|e| {
        e.string()
            .map(|s| s.trim())
            .map_err(|error| DicomTagReadError::NotString { error, tag })
    })
}

pub(crate) fn tts(dcm: &DefaultDicomObject, tag: Tag) -> Result<String, DicomTagReadError> {
    tt(dcm, tag).map(|s| s.replace('\0', ""))
}

/// Replace disallowed characters with "_".
/// https://github.com/FNNDSC/pypx/blob/7619c15f4d2303d6d5ca7c255d81d06c7ab8682b/pypx/repack.py#L424
///
/// Also, it's necessary to handle NUL bytes...
pub(crate) fn sanitize<S: AsRef<str>>(s: S) -> String {
    let s_nonull = s.as_ref().replace('\0', "");
    VALID_CHARS_RE
        .get_or_init(|| Regex::new(r#"[^A-Za-z0-9\.\-]+"#).unwrap())
        .replace_all(&s_nonull, "_")
        .to_string()
}

static VALID_CHARS_RE: OnceLock<Regex> = OnceLock::new();
