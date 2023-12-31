use crate::log_write::write_logs;
use crate::pack_path::PypxPath;
use camino::{Utf8Path, Utf8PathBuf};

use crate::dicom_data::DicomTagAndError;
use std::path::Path;

pub fn repack(
    dicom_file: &Utf8Path,
    data_dir: &Utf8Path,
    log_dir: Option<&Utf8Path>,
    cleanup: bool,
) -> anyhow::Result<RepackOutcome> {
    let dcm = dicom::object::open_file(dicom_file)?;
    let common = (&dcm).try_into()?;
    let unpack = PypxPath::new(&common, data_dir);

    fs_err::create_dir_all(&unpack.dir)?;
    copy_or_mv(dicom_file, &unpack.path, cleanup)?;

    let missing = if let Some(d) = log_dir {
        write_logs(&dcm, &common, &unpack, d)?
    } else {
        Vec::new()
    };
    let outcome = RepackOutcome {
        dst: unpack.path,
        missing,
        PatientID: common.PatientID.to_string(),
        SeriesInstanceUID: common.SeriesInstanceUID,
    };
    anyhow::Ok(outcome)
}

/// Information about what the function [repack] did, for logging purposes.
#[allow(non_snake_case)]
pub struct RepackOutcome {
    pub dst: Utf8PathBuf,
    pub missing: Vec<DicomTagAndError>,
    pub PatientID: String,
    pub SeriesInstanceUID: String,
}

fn copy_or_mv<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
    cleanup: bool,
) -> std::io::Result<()> {
    if cleanup {
        mv(&src, &dst)?;
    } else {
        fs_err::copy(src, dst)?;
    }

    Ok(())
}

/// Rename a file.
fn mv<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> std::io::Result<()> {
    if fs_err::rename(&src, &dst).is_ok() {
        return Ok(());
    }
    // std::fs::rename is efficient, but will fail when src and dst are on different mount points
    // https://doc.rust-lang.org/std/fs/fn.rename.html
    fs_err::copy(&src, &dst).and_then(|_| fs_err::remove_file(src))
}

#[cfg(test)]
mod test {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_copy() {
        let tempdir = TempDir::new("repack_unit_test").unwrap();
        let src = tempdir.path().join("favorite_drink.txt");
        let dst = tempdir.path().join("destination.txt");
        fs_err::write(&src, "i enjoy bubble tea").unwrap();
        copy_or_mv(&src, &dst, false).unwrap();

        let original_data =
            fs_err::read_to_string(src).expect("Could not read src file, could it be missing?");
        let copied_data =
            fs_err::read_to_string(dst).expect("Could not read dst file, could it be missing?");
        assert_eq!(original_data, copied_data)
    }

    #[test]
    fn test_mv() {
        let tempdir = TempDir::new("repack_unit_test").unwrap();
        let src = tempdir.path().join("favorite_drink.txt");
        let dst = tempdir.path().join("destination.txt");
        let data = "i enjoy bubble tea";
        fs_err::write(&src, &data).unwrap();
        copy_or_mv(&src, &dst, true).unwrap();

        let copied_data =
            fs_err::read_to_string(dst).expect("Could not read dst file, could it be missing?");
        assert_eq!(data, &copied_data);
        assert!(!src.exists())
    }
}
