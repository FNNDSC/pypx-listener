use crate::dicom_info::DicomInfo;
use crate::log_models::PatientData;
use camino::Utf8Path;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

/// Write stuff to `/home/dicom/log/{patientData,seriesData,studyData}`
pub(crate) fn write_logs(
    info: DicomInfo,
    log_dir: &Utf8Path,
    pack_dir: &Utf8Path,
    fname: &str,
) -> io::Result<()> {
    let patient_data_dir = log_dir.join("patientDatta");
    let series_data_dir = log_dir.join("seriesData");
    let study_data_dir = log_dir.join("studyData");

    let patient_data_fname = patient_data_dir
        .join(&info.PatientID)
        .with_extension("json");
    let mut patient_data: PatientData =
        load_json_carelessly(&patient_data_fname).unwrap_or_else(|| (&info).into());
    patient_data
        .StudyList
        .insert(info.StudyInstanceUID.to_string());
    write_json(patient_data, patient_data_fname)?;

    Ok(())
}

/// Read and deserialize a JSON file. In case of any error, return `None`.
///
/// Some possible errors:
/// - File does not exist
/// - JSON data not well formed or not valid
fn load_json_carelessly<P: Into<PathBuf>, D: DeserializeOwned>(p: P) -> Option<D> {
    let file = fs_err::File::open(p).ok()?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader).ok()?;
    Some(data)
}

/// Write data to a JSON file. Will create parent directories as needed.
fn write_json<S: Serialize, P: AsRef<Utf8Path>>(data: S, p: P) -> io::Result<()> {
    if let Some(parent) = p.as_ref().parent() {
        fs_err::create_dir_all(parent)?;
    }
    let file = fs_err::File::create(p.as_ref())?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &data).unwrap();
    Ok(())
}
