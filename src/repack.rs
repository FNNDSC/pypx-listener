use crate::log_write::write_logs;
use crate::pack_path::{PypxPath, PypxPathElements};
use camino::Utf8Path;
use dicom::core::header::Header;
use std::path::Path;
use anyhow::bail;

pub fn repack(
    dicom_file: &Utf8Path,
    data_dir: &Utf8Path,
    log_dir: Option<&Utf8Path>,
    cleanup: bool,
) -> anyhow::Result<()> {
    let dcm = dicom::object::open_file(dicom_file)?;
    let elements = (&dcm).try_into()?;
    let unpack = PypxPath::new(&elements, data_dir);

    fs_err::create_dir_all(&unpack.dir)?;
    fs_err::copy(dicom_file, &unpack.path)?;
    // copy_or_mv(dicom_file, &unpack.path, cleanup)?;  // not working, figure out why later
    eprintln!("{} -> {}", dicom_file, &unpack.path);
    if !unpack.path.is_file() {  // delete me later
        bail!("Copy failed, {} does not exist.", &unpack.path)
    }

    if let Some(d) = log_dir {
        write_logs(&dcm, &elements, &unpack, d)?;
    }

    anyhow::Ok(())
}

fn copy_or_mv<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
    cleanup: bool,
) -> std::io::Result<()> {
    if cleanup {
        mv(src, dst)?;
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
    fs_err::copy(&src, &dst).and_then(|_| fs_err::remove_file(dst))
}
