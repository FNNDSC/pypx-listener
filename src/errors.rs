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
