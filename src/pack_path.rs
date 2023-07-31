//! Functions for deciding where to copy the received DICOM to.
use crate::errors::DicomTagReadError;
use crate::helpers::{sanitize, tt, tts};
use camino::{Utf8Path, Utf8PathBuf};
use dicom::dictionary_std::tags;
use dicom::object::DefaultDicomObject;

/// Destination directory and file name for the DICOM file.
pub(crate) struct PypxPath {
    pub path: Utf8PathBuf,
    pub dir: Utf8PathBuf,
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
        let pack_dir = data_dir.join(pack_dir_rel);
        let path = pack_dir.join(&fname);
        Self {
            fname,
            dir: pack_dir,
            path,
        }
    }
}

/// DICOM elements which a [PypxPath] is comprised of.
#[allow(non_snake_case)]
pub(crate) struct PypxPathElements<'a> {
    // these are all part of the path name.
    pub InstanceNumber: &'a str,
    pub SOPInstanceUID: &'a str,
    pub PatientID: &'a str,
    pub PatientName: &'a str,
    pub PatientBirthDate: &'a str,
    pub StudyDescription: &'a str,
    pub AccessionNumnber: &'a str,
    pub StudyDate: &'a str,
    pub SeriesNumber: u32,
    pub SeriesDescription: &'a str,

    // these are not part of the path name, but used in the log path names.
    pub StudyInstanceUID: String,
    pub SeriesInstanceUID: String,
}

impl<'a> TryFrom<&'a DefaultDicomObject> for PypxPathElements<'a> {
    type Error = DicomTagReadError;
    fn try_from(dcm: &'a DefaultDicomObject) -> Result<Self, Self::Error> {
        // NOTE: the implementation here is optimized based on implementation details of dicom-rs v0.5.4.
        // - dcm.element(...)?.string() produces a reference to the data w/o cloning nor parsing
        // - dcm.element is more efficient than dcm.element_by_name, sinnce the latter does a map lookup
        let data = Self {
            InstanceNumber: tt(dcm, tags::INSTANCE_NUMBER)?,
            SOPInstanceUID: tt(dcm, tags::SOP_INSTANCE_UID)?,
            PatientID: tt(dcm, tags::PATIENT_ID)?,
            PatientName: tt(dcm, tags::PATIENT_NAME)?,
            PatientBirthDate: tt(dcm, tags::PATIENT_BIRTH_DATE)?,
            StudyDescription: tt(dcm, tags::STUDY_DESCRIPTION)?,
            AccessionNumnber: tt(dcm, tags::ACCESSION_NUMBER)?,
            StudyDate: tt(dcm, tags::STUDY_DATE)?,
            SeriesNumber: tt(dcm, tags::SERIES_NUMBER)?.parse()?,
            SeriesDescription: tt(dcm, tags::SERIES_DESCRIPTION)?,
            StudyInstanceUID: tts(dcm, tags::STUDY_INSTANCE_UID)?,
            SeriesInstanceUID: tts(dcm, tags::SERIES_INSTANCE_UID)?,
        };
        Ok(data)
    }
}
