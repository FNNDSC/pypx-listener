use std::num::ParseIntError;

/// Error reading a DICOM tag's value.
#[derive(thiserror::Error, Debug)]
pub(crate) enum DicomTagReadError {
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    NotFound(#[from] dicom::object::Error),
    #[error("{tag:?} tag value is not a string")]
    NotString {
        #[source]
        error: dicom::core::value::CastValueError,
        tag: dicom::core::Tag,
    },
}

/// Error decoding a DICOM tag's value.
#[derive(thiserror::Error, Debug)]
pub(crate) enum DicomElementSerializationError {
    #[error("Unknown tag: {0}")]
    UnknownTagError(dicom::core::Tag),
    #[error("{0} should not be serialized.")]
    Excluded(dicom::core::Tag),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    CastValueError(#[from] dicom::core::value::CastValueError),
}
