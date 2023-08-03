//! Functions for deciding where to copy the received DICOM to.
use crate::dicom_data::CommonElements;
use crate::helpers::sanitize;
use camino::{Utf8Path, Utf8PathBuf};

/// Destination directory and file name for the DICOM file.
pub(crate) struct PypxPath {
    pub path: Utf8PathBuf,
    pub dir: Utf8PathBuf,
    pub fname: String,
}

impl PypxPath {
    /// Equivalent Python implementation `pypx.repack.Process.packPath_resolve`:
    /// https://github.com/FNNDSC/pypx/blob/d4791598f65b257cbf6b17d6b5b05db777844db4/pypx/repack.py#L412-L459
    ///
    /// Missing DICOM element values are replaced with the name of the DICOM tag.
    /// See https://github.com/FNNDSC/pypx/wiki/How-pypx-handles-missing-elements
    pub fn new(dcm: &CommonElements, data_dir: &Utf8Path) -> Self {
        let root_dir = sanitize(format!(
            "{}-{}-{}",
            dcm.PatientID,
            dcm.PatientName.unwrap_or("PatientName"),
            dcm.PatientBirthDate.unwrap_or("PatientBirthDate")
        ));
        let study_dir = sanitize(format!(
            "{}-{}-{}",
            dcm.StudyDescription.unwrap_or("StudyDescription"),
            dcm.AccessionNumnber,
            dcm.StudyDate.unwrap_or("StudyDate")
        ));
        let series_dir = sanitize(format!(
            "{:0>5}-{}",
            dcm.SeriesNumber,
            dcm.SeriesDescription.unwrap_or("SeriesDescription")
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
