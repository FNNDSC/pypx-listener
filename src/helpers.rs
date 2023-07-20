use crate::errors::DicomTagReadError;
use dicom::core::Tag;
use dicom::object::DefaultDicomObject;

/// Get the `&str` to a DICOM object.
///
/// I tried to make this helper function low-cost.
pub(crate) fn tt(dcm: &DefaultDicomObject, tag: Tag) -> Result<&str, DicomTagReadError> {
    dcm.element(tag).map_err(|e| e.into()).and_then(|e| {
        e.string()
            .map(|s| s.trim())
            .map_err(|error| DicomTagReadError::NotString { error, tag })
    })
}
