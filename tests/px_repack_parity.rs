extern crate tempdir;

use anyhow::{bail, Context};
use rx_repack::repack;
use std::collections::HashMap;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempdir::TempDir;

const EXAMPLES_DIR: &str = "./examples/";

#[test]
fn test_parity_with_px_repack() -> anyhow::Result<()> {
    let (od_dir, expected_data_dir, expected_log_dir) =
        find_examples().with_context(examples_instructions)?;

    let tmp_dir = TempDir::new("example")?;
    let tmp_path = tmp_dir.path();
    // let tmp_path = Path::new("./test_output");
    let actual_data_dir = tmp_path.join("data");
    let actual_log_dir = tmp_path.join("log");

    process_all(&od_dir, &actual_data_dir, &actual_log_dir)?;
    assert!(dirs_are_equal(&expected_data_dir, &actual_data_dir));

    let expected_patient_data_dir = expected_log_dir.join("patientData");
    let actual_patient_data_dir = actual_log_dir.join("patientData");
    let patient_data_pairs = file_by_file(&expected_patient_data_dir, &actual_patient_data_dir);
    for (expected_file, actual_file) in patient_data_pairs {
        assert!(actual_file.is_file());
        assert_json_equal(&expected_file, &actual_file);
    }

    anyhow::Ok(())
}

/// Run rx-repack on all files in a directory.
fn process_all(od_dir: &Path, data_dir: &Path, log_dir: &Path) -> anyhow::Result<()> {
    fs_err::read_dir(od_dir)
        .unwrap()
        .map(|r| r.unwrap())
        .filter(|e| e.metadata().unwrap().is_file())
        .filter(|e| e.file_name().to_str().unwrap_or("").ends_with(".dcm"))
        .map(|e| e.path())
        .map(|dicom_file| (repack(dicom_file, data_dir, Some(log_dir), false)))
        .collect()
}

fn dirs_are_equal(expected: &Path, actual: &Path) -> bool {
    Command::new("diff")
        .arg("-r")
        .arg(expected)
        .arg(actual)
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
}

fn assert_json_equal(expected: &Path, actual: &Path) {
    assert_eq!(
        load_json(expected),
        load_json(actual),
        "JSON file {:?} not the same as {:?}",
        expected,
        actual
    )
}

fn load_json<P: AsRef<Path>>(p: P) -> HashMap<String, String> {
    let file = fs_err::File::open(p.as_ref()).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

fn file_by_file<'a>(
    expected: &'a Path,
    actual: &'a Path,
) -> impl Iterator<Item = (PathBuf, PathBuf)> + 'a {
    let s = expected.join("**").into_os_string();
    let p = s.to_str().unwrap();
    glob::glob(p)
        .unwrap()
        .map(|r| r.unwrap())
        .map(move |expected_path| {
            let rel = pathdiff::diff_paths(&expected_path, expected).unwrap();
            let actual_path = actual.join(rel);
            (expected_path, actual_path)
        })
}

fn examples_instructions() -> String {
    format!(
        "Please run:\n\n\trm -rf {}\n\t./get_examples.sh {}",
        &EXAMPLES_DIR, &EXAMPLES_DIR
    )
}

fn find_examples() -> anyhow::Result<(PathBuf, PathBuf, PathBuf)> {
    let mut input_dir: Option<PathBuf> = None;
    let mut log_dir: Option<PathBuf> = None;
    let mut data_dir: Option<PathBuf> = None;
    let read_dir = fs_err::read_dir(EXAMPLES_DIR)?
        .filter_map(|r| r.ok())
        .map(|e| e.path());
    for p in read_dir {
        if file_name_starts_with(&p, "FNNDSC-SAG-anon-") {
            input_dir = Some(p.clone());
        } else if file_name_starts_with(&p, "px-repack-output") {
            data_dir = Some(p.join("data"));
            log_dir = Some(p.join("log"));
            for dir in (&[&data_dir, &log_dir]).iter().filter_map(|f| f.as_ref()) {
                if !dir.is_dir() {
                    bail!("{:?} is not a directory", dir);
                }
            }
        }
    }
    input_dir
        .zip(data_dir)
        .zip(log_dir)
        .map(|((i, d), l)| (i, d, l))
        .ok_or_else(|| anyhow::Error::msg("Examples not found"))
}

fn file_name_starts_with(p: &Path, prefix: &str) -> bool {
    p.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.starts_with(prefix))
        .unwrap_or(false)
}
