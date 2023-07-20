/// Get the `&str` to a DICOM element.
#[macro_export]
macro_rules! tt {
    ($dcm:expr, $tag:expr) => {
        $dcm.element($tag).map_err(|e| e.into()).and_then(|e| {
            e.string()
                .map(|s| s.trim())
                .map_err(|error| $crate::errors::DicomTagReadError::NotString { error, tag: $tag })
        })
    };
}
