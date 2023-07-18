//! DICOM tag extraction.
use camino::Utf8PathBuf;
use dicom::dictionary_std::tags;
use regex::Regex;
use std::borrow::Cow;
use std::sync::OnceLock;

#[allow(non_snake_case)]
pub(crate) struct DicomInfo {
    pub PatientID: String,
    pub PatientName: String,
    pub PatientAge: String,
    pub PatientSex: char,
    pub PatientBirthDate: String,
    pub AccessionNumber: String,
    pub StudyInstanceUID: String,
    pub StudyDescription: String,
    pub StudyDate: String,
    pub SeriesNumber: u32,
    pub SeriesDescription: String,
    pub InstanceNumber: u32,
    pub SOPInstanceUID: String,
}

fn es(
    dcm: &dicom::object::DefaultDicomObject,
    tag: dicom::core::Tag,
) -> Result<Cow<str>, dicom::object::Error> {
    dcm.element(tag).map(|data| data.to_str().unwrap())
}

impl TryFrom<dicom::object::DefaultDicomObject> for DicomInfo {
    type Error = dicom::object::Error;

    fn try_from(dcm: dicom::object::DefaultDicomObject) -> Result<Self, Self::Error> {
        let info = Self {
            PatientID: es(&dcm, tags::PATIENT_ID)?.to_string(),
            PatientName: es(&dcm, tags::PATIENT_NAME)?.to_string(),
            PatientAge: es(&dcm, tags::PATIENT_AGE)?.to_string(),
            PatientSex: es(&dcm, tags::PATIENT_SEX)?.parse().unwrap(),
            PatientBirthDate: es(&dcm, tags::PATIENT_BIRTH_DATE)?.to_string(),
            StudyDescription: es(&dcm, tags::STUDY_DESCRIPTION)?.to_string(),
            AccessionNumber: es(&dcm, tags::ACCESSION_NUMBER)?.to_string(),
            StudyInstanceUID: es(&dcm, tags::STUDY_INSTANCE_UID)?.to_string(),
            StudyDate: es(&dcm, tags::STUDY_DATE)?.to_string(),
            SeriesNumber: es(&dcm, tags::SERIES_NUMBER)?.parse().unwrap(),
            SeriesDescription: es(&dcm, tags::SERIES_DESCRIPTION)?.to_string(),
            InstanceNumber: es(&dcm, tags::INSTANCE_NUMBER)?.parse().unwrap(),
            SOPInstanceUID: es(&dcm, tags::SOP_INSTANCE_UID)?.to_string(),
        };
        Ok(info)
    }
}

impl DicomInfo {
    /// Produce the destination directory and file name for the DICOM file.
    /// Equivalent Python implementation is `pypx.repack.Process.packPath_resolve`
    /// https://github.com/FNNDSC/pypx/blob/d4791598f65b257cbf6b17d6b5b05db777844db4/pypx/repack.py#L412-L459
    pub(crate) fn to_path_parts(&self) -> (Utf8PathBuf, String) {
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
            Utf8PathBuf::from(root_dir.as_ref())
                .join(study_dir.as_ref())
                .join(series_dir.as_ref()),
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
