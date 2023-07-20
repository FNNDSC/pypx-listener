extern crate tempdir;

use anyhow::{bail, Context};
use camino::{Utf8Path, Utf8PathBuf};
use rx_repack::repack;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;
use tempdir::TempDir;

const EXAMPLES_DIR: &str = "examples/";

#[test]
fn test_parity_with_px_repack() -> anyhow::Result<()> {
    let (od_dir, expected_data_dir, expected_log_dir) =
        find_examples().with_context(examples_instructions)?;

    let tmp_dir = TempDir::new("example")?;
    let tmp_path = Utf8Path::from_path(tmp_dir.path()).unwrap();
    // let tmp_path = Utf8Path::new("./test_output");
    let actual_data_dir = tmp_path.join("data");
    let actual_log_dir = tmp_path.join("log");

    process_all(&od_dir, &actual_data_dir, &actual_log_dir)?;
    assert!(dirs_are_equal(&expected_data_dir, &actual_data_dir));

    let expected_patient_data_dir = expected_log_dir.join("patientData");
    let actual_patient_data_dir = actual_log_dir.join("patientData");
    let patient_data_pairs = file_by_file(&expected_patient_data_dir, &actual_patient_data_dir);
    for (expected_file, actual_file) in patient_data_pairs {
        assert!(
            actual_file.is_file(),
            "{} is not a file. Parent has files: {:?}",
            &actual_file,
            glob::glob(&actual_file.with_file_name("*").as_str())
                .unwrap()
                .map(|r| r
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or("INVALID".to_string()))
                .collect::<Vec<String>>()
        );
        assert_json_equal(&expected_file, &actual_file);
        println!("OK");
    }

    // assert all files present, w/o checking their contents (for now)
    for (expected_file, actual_file) in file_by_file(&expected_log_dir, &actual_log_dir) {
        assert!(
            actual_file.is_file(),
            "{} is not a file. Parent has files: {:?}",
            &actual_file,
            glob::glob(&actual_file.with_file_name("*").as_str())
                .unwrap()
                .map(|r| r
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or("INVALID".to_string()))
                .collect::<Vec<String>>()
        );
    }

    anyhow::Ok(())
}

/// Run rx-repack on all files in a directory.
fn process_all(od_dir: &Utf8Path, data_dir: &Utf8Path, log_dir: &Utf8Path) -> anyhow::Result<()> {
    fs_err::read_dir(od_dir)
        .unwrap()
        .map(|r| r.unwrap())
        .filter(|e| e.metadata().unwrap().is_file())
        .filter(|e| e.file_name().to_str().unwrap_or("").ends_with(".dcm"))
        .map(|e| e.path())
        .map(Utf8PathBuf::from_path_buf)
        .map(Result::unwrap)
        .map(|dicom_file| (repack(&dicom_file, data_dir, Some(log_dir), false)))
        .collect()
}

fn dirs_are_equal(expected: &Utf8Path, actual: &Utf8Path) -> bool {
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

fn assert_json_equal(expected: &Utf8Path, actual: &Utf8Path) {
    assert_eq!(
        load_json(expected),
        load_json(actual),
        "JSON file {:?} not the same as {:?}",
        expected,
        actual
    )
}

fn load_json<P: AsRef<Path>>(p: P) -> serde_json::Value {
    let file = fs_err::File::open(p.as_ref()).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

fn file_by_file<'a>(
    expected: &'a Utf8Path,
    actual: &'a Utf8Path,
) -> impl Iterator<Item = (Utf8PathBuf, Utf8PathBuf)> + 'a {
    let s = expected.join("**/*").into_os_string();
    let p = s.to_str().unwrap();
    glob::glob(p)
        .unwrap()
        .map(|r| r.unwrap())
        .map(Utf8PathBuf::from_path_buf)
        .map(Result::unwrap)
        .filter(|p| p.is_file())
        .map(move |expected_path| {
            let rel = pathdiff::diff_utf8_paths(&expected_path, expected).unwrap();
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

fn find_examples() -> anyhow::Result<(Utf8PathBuf, Utf8PathBuf, Utf8PathBuf)> {
    let mut input_dir: Option<Utf8PathBuf> = None;
    let mut log_dir: Option<Utf8PathBuf> = None;
    let mut data_dir: Option<Utf8PathBuf> = None;
    let read_dir = fs_err::read_dir(EXAMPLES_DIR)?
        .filter_map(|r| r.ok())
        .map(|e| e.path())
        .map(Utf8PathBuf::from_path_buf)
        .map(Result::unwrap);
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

fn file_name_starts_with(p: &Utf8Path, prefix: &str) -> bool {
    p.file_name()
        .map(|s| s.starts_with(prefix))
        .unwrap_or(false)
}
