use crate::dicom_info::DicomInfo;
use crate::log_write::write_logs;
use camino::Utf8Path;
use std::path::Path;

pub fn repack(
    dicom_file: &Utf8Path,
    data_dir: &Utf8Path,
    log_dir: Option<&Utf8Path>,
    cleanup: bool,
) -> anyhow::Result<()> {
    let dcm = dicom::object::open_file(dicom_file)?;
    let dicom_info = DicomInfo::try_from(dcm)?;
    let pack_dir_rel = dicom_info.pack_path();
    let pack_dir = data_dir.join(pack_dir_rel);

    copy_or_mv(dicom_file, &pack_dir, &dicom_info.pypx_fname, cleanup)?;

    if let Some(d) = log_dir {
        write_logs(&dicom_info, d, &pack_dir)?;
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
