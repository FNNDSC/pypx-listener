use crate::log_write::write_logs;
use crate::pack_path::{PypxPath, PypxPathElements};
use camino::Utf8Path;
use dicom::core::header::Header;
use std::path::Path;

pub fn repack(
    dicom_file: &Utf8Path,
    data_dir: &Utf8Path,
    log_dir: Option<&Utf8Path>,
    cleanup: bool,
) -> anyhow::Result<()> {
    let dcm = dicom::object::open_file(dicom_file)?;
    let elements = (&dcm).try_into()?;
    let unpack = PypxPath::new(&elements, data_dir);

    copy_or_mv(dicom_file, &unpack.dir, &unpack.path, cleanup)?;

    if let Some(d) = log_dir {
        write_logs(&dcm, &elements, &unpack, d)?;
    }

    anyhow::Ok(())
}

fn copy_or_mv<P: AsRef<Path>, Q: AsRef<Path>, R: AsRef<Path>>(
    src: P,
    dst_dir: Q,
    fname: R,
    cleanup: bool,
) -> std::io::Result<()> {
    fs_err::create_dir_all(&dst_dir)?;
    let dst = dst_dir.as_ref().join(fname);
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
