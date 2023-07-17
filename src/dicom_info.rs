//! DICOM tag extraction.
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::OnceLock;
use dicom::dictionary_std::tags;
use regex::Regex;

#[allow(non_snake_case)]
pub(crate) struct DicomInfo {
    PatientID: String,
    PatientName: String,
    PatientBirthDate: String,
    StudyDescription: String,
    AccessionNumber: String,
    StudyDate: String,
    SeriesNumber: u32,
    SeriesDescription: String,
    InstanceNumber: u32,
    SOPInstanceUID: String,
}

impl TryFrom<dicom::object::DefaultDicomObject> for DicomInfo {
    type Error = dicom::object::Error;

    fn try_from(dcm: dicom::object::DefaultDicomObject) -> Result<Self, Self::Error> {
        let info = Self {
            PatientID: dcm.element(tags::PATIENT_ID)?.to_str().unwrap().to_string(),
            PatientName: dcm
                .element(tags::PATIENT_NAME)?
                .to_str()
                .unwrap()
                .to_string(),
            PatientBirthDate: dcm
                .element(tags::PATIENT_BIRTH_DATE)?
                .to_str()
                .unwrap()
                .to_string(),
            StudyDescription: dcm
                .element(tags::STUDY_DESCRIPTION)?
                .to_str()
                .unwrap()
                .to_string(),
            AccessionNumber: dcm
                .element(tags::ACCESSION_NUMBER)?
                .to_str()
                .unwrap()
                .to_string(),
            StudyDate: dcm.element(tags::STUDY_DATE)?.to_str().unwrap().to_string(),
            SeriesNumber: dcm
                .element(tags::SERIES_NUMBER)?
                .to_str()
                .unwrap()
                .parse()
                .unwrap(),
            SeriesDescription: dcm
                .element(tags::SERIES_DESCRIPTION)?
                .to_str()
                .unwrap()
                .to_string(),
            InstanceNumber: dcm
                .element(tags::INSTANCE_NUMBER)?
                .to_str()
                .unwrap()
                .parse()
                .unwrap(),
            SOPInstanceUID: dcm
                .element(tags::SOP_INSTANCE_UID)?
                .to_str()
                .unwrap()
                .to_string(),
        };
        Ok(info)
    }
}

impl DicomInfo {
    /// Produce the destination directory and file name for the DICOM file.
    /// Equivalent Python implementation is `pypx.repack.Process.packPath_resolve`
    /// https://github.com/FNNDSC/pypx/blob/d4791598f65b257cbf6b17d6b5b05db777844db4/pypx/repack.py#L412-L459
    pub(crate) fn to_path_parts(&self) -> (PathBuf, String) {
        let root_string = format!(
            "{}-{}-{}",
            &self.PatientID, &self.PatientName, &self.PatientBirthDate
        );
        let study_string = format!(
            "{}-{}-{}",
            &self.StudyDescription, &self.AccessionNumber, &self.StudyDate
        );
        let series_string = format!("{:0>5}-{}", &self.SeriesNumber, &self.SeriesDescription);
        let image_string = format!("{:0>4}-{}.dcm", &self.InstanceNumber, &self.SOPInstanceUID);

        let root_dir = sanitize(&root_string);
        let study_dir = sanitize(&study_string);
        let series_dir = sanitize(&series_string);
        let image_file = sanitize(&image_string);

        (
            PathBuf::from(root_dir.as_ref())
                .join(&study_dir.as_ref())
                .join(&series_dir.as_ref()),
            image_file.to_string(),
        )
    }
}

/// Replace disallowed characters with "_"
/// https://github.com/FNNDSC/pypx/blob/7619c15f4d2303d6d5ca7c255d81d06c7ab8682b/pypx/repack.py#L424
fn sanitize(s: &str) -> Cow<str> {
    VALID_CHARS_RE
        .get_or_init(|| Regex::new(r#"[^A-Za-z0-9\.\-]+"#).unwrap())
        .replace_all(s, "_")
}

static VALID_CHARS_RE: OnceLock<Regex> = OnceLock::new();
