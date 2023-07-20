//! Functions for deciding where to copy the received DICOM to.
use crate::errors::DicomTagReadError;
use crate::helpers::tt;
use camino::{Utf8Path, Utf8PathBuf};
use dicom::dictionary_std::tags;
use dicom::object::{DefaultDicomObject, Tag};
use regex::Regex;
use std::borrow::Cow;
use std::sync::OnceLock;

/// Destination directory and file name for the DICOM file.
pub(crate) struct PypxPath {
    pub path: Utf8PathBuf,
    pub dir: Utf8PathBuf,
    pub dir_rel: Utf8PathBuf,
    pub fname: String,
}

impl PypxPath {
    /// Equivalent Python implementation `pypx.repack.Process.packPath_resolve`:
    /// https://github.com/FNNDSC/pypx/blob/d4791598f65b257cbf6b17d6b5b05db777844db4/pypx/repack.py#L412-L459
    pub fn new(dcm: &PypxPathElements, data_dir: &Utf8Path) -> Self {
        let root_dir = sanitize(format!(
            "{}-{}-{}",
            dcm.PatientID, dcm.PatientName, dcm.PatientBirthDate
        ));
        let study_dir = sanitize(format!(
            "{}-{}-{}",
            dcm.StudyDescription, dcm.AccessionNumnber, dcm.StudyDate
        ));
        let series_dir = sanitize(format!(
            "{:0>5}-{}",
            dcm.SeriesNumber, dcm.SeriesDescription
        ));

        let pack_dir_rel = Utf8PathBuf::from(root_dir).join(study_dir).join(series_dir);
        let fname = sanitize(format!(
            "{:0>4}-{}.dcm",
            dcm.InstanceNumber, dcm.SOPInstanceUID
        ));
        let pack_dir = data_dir.join(&pack_dir_rel);
        let path = pack_dir.join(&fname);
        Self {
            fname,
            dir_rel: pack_dir_rel,
            dir: pack_dir,
            path,
        }
    }
}

/// DICOM elements which a [PypxPath] is comprised of.
#[allow(non_snake_case)]
pub(crate) struct PypxPathElements<'a> {
    pub InstanceNumber: &'a str,
    pub SOPInstanceUID: &'a str,
    pub PatientID: &'a str,
    pub PatientName: &'a str,
    pub PatientBirthDate: &'a str,
    pub StudyDescription: &'a str,
    pub AccessionNumnber: &'a str,
    pub StudyDate: &'a str,
    pub SeriesNumber: &'a str,
    pub SeriesDescription: &'a str,
}

impl<'a> TryFrom<&'a DefaultDicomObject> for PypxPathElements<'a> {
    type Error = DicomTagReadError;
    fn try_from(dcm: &'a DefaultDicomObject) -> Result<Self, Self::Error> {
        // NOTE: the implementation here is optimized based on implementation details of dicom-rs v0.5.4.
        // - dcm.element(...)?.string() produces a reference to the data w/o cloning nor parsing
        // - dcm.element is more efficient than dcm.element_by_name, sinnce the latter does a map lookup
        let data = Self {
            InstanceNumber: tt(&dcm, tags::INSTANCE_NUMBER)?,
            SOPInstanceUID: tt(&dcm, tags::SOP_INSTANCE_UID)?,
            PatientID: tt(&dcm, tags::PATIENT_ID)?,
            PatientName: tt(&dcm, tags::PATIENT_NAME)?,
            PatientBirthDate: tt(&dcm, tags::PATIENT_BIRTH_DATE)?,
            StudyDescription: tt(&dcm, tags::STUDY_DESCRIPTION)?,
            AccessionNumnber: tt(&dcm, tags::ACCESSION_NUMBER)?,
            StudyDate: tt(&dcm, tags::STUDY_DATE)?,
            SeriesNumber: tt(&dcm, tags::SERIES_NUMBER)?,
            SeriesDescription: tt(&dcm, tags::SERIES_DESCRIPTION)?,
        };
        Ok(data)
    }
}

/// Replace disallowed characters with "_".
/// https://github.com/FNNDSC/pypx/blob/7619c15f4d2303d6d5ca7c255d81d06c7ab8682b/pypx/repack.py#L424
///
/// Also, it's necessary to handle NUL bytes...
fn sanitize<S: AsRef<str>>(s: S) -> String {
    let s_nonull = s.as_ref().replace('\0', "");
    VALID_CHARS_RE
        .get_or_init(|| Regex::new(r#"[^A-Za-z0-9\.\-]+"#).unwrap())
        .replace_all(&s_nonull, "_")
        .to_string()
}

static VALID_CHARS_RE: OnceLock<Regex> = OnceLock::new();
