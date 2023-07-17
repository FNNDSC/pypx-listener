
use std::path::Path;
use crate::dicom_info::DicomInfo;

pub fn repack<P: AsRef<Path>, Q: AsRef<Path>, R: AsRef<Path>>(
    dicom_file: P,
    data_dir: Q,
    log_dir: Option<R>,
    cleanup: bool,
) -> anyhow::Result<()> {
    let dcm = dicom::object::open_file(&dicom_file)?;
    let dicom_info = DicomInfo::try_from(dcm)?;
    let (pack_dir_rel, fname) = dicom_info.to_path_parts();
    let pack_dir = data_dir.as_ref().join(pack_dir_rel);

    copy_or_mv(&dicom_file, &pack_dir, &fname, cleanup)?;

    anyhow::Ok(())
}

fn copy_or_mv<P: AsRef<Path>, Q: AsRef<Path>, R: AsRef<Path>>(
    src: P,
    dst_dir: Q,
    fname: R,
    cleanup: bool,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst_dir)?;
    let dst = dst_dir.as_ref().join(fname);
    if cleanup {
        mv(src, dst)?;
    } else {
        std::fs::copy(src, dst)?;
    }
    Ok(())
}

/// Rename a file.
fn mv<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> std::io::Result<()> {
    if std::fs::rename(&src, &dst).is_ok() {
        return Ok(());
    }
    std::fs::copy(&src, &dst).and_then(|_| std::fs::remove_file(dst))
}
