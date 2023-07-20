/// Error reading a DICOM tag's value.
#[derive(thiserror::Error, Debug)]
pub(crate) enum DicomTagReadError {
    #[error(transparent)]
    NotFound(#[from] dicom::object::Error),
    #[error("{tag:?} tag value is not a string")]
    NotString {
        #[source]
        error: dicom::core::value::CastValueError,
        tag: dicom::core::Tag,
    },
}
