use crate::log_models::*;
use crate::pack_path::PypxPath;
use camino::Utf8Path;

use crate::dicom_data::{CommonElements, DicomTagAndError, TagExtractor};
use crate::serialize_seriesmeta::StudyDataSeriesMeta;
use dicom::object::DefaultDicomObject;
use hashbrown::HashMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

/// Write *pypx* "stuff" to `/home/dicom/log/{patientData,seriesData,studyData}`.
/// The "stuff" is read by downstream _pypx_ programs such as `px-register`, `px-status`.
#[allow(non_snake_case)]
pub(crate) fn write_logs(
    dcm: &DefaultDicomObject,
    common: &CommonElements,
    unpack: &PypxPath,
    log_dir: &Utf8Path,
) -> anyhow::Result<Vec<DicomTagAndError>> {
    let dcmtags = TagExtractor::new(dcm);
    let patient_data_dir = log_dir.join("patientData");
    let series_data_dir = log_dir.join("seriesData");
    let study_data_dir = log_dir.join("studyData");

    // write stuff to patientData/MRN.json
    let patient_data_fname = patient_data_dir
        .join(common.PatientID)
        .with_extension("json");
    let mut patient_data: HashMap<String, PatientData> =
        load_json_carelessly(&patient_data_fname).unwrap_or_else(|| HashMap::with_capacity(1));
    patient_data
        .entry_ref(common.PatientID)
        .or_insert_with(|| PatientData::new(&dcmtags, common))
        .StudyList
        .insert(common.StudyInstanceUID.to_string());
    write_json(patient_data, patient_data_fname)?;

    // write stuff to studyData/X.X.X.XXXXX-series/Y.Y.Y.YYYYY-meta.json
    let study_series_meta_dir = study_data_dir.join(format!("{}-series", &common.StudyInstanceUID));
    fs_err::create_dir_all(&study_series_meta_dir)?;
    let study_series_meta_fname =
        study_series_meta_dir.join(format!("{}-meta.json", &common.SeriesInstanceUID));
    if !study_series_meta_fname.is_file() {
        let study_series_meta =
            StudyDataSeriesMeta::new(&common.SeriesInstanceUID, unpack.dir.as_str(), dcm);
        let data: HashMap<_, _> = [(&common.StudyInstanceUID, study_series_meta)].into();
        write_json(data, study_series_meta_fname)?;
    }

    // write stuff to studyData/X.X.X.XXXXX-meta.json
    let study_meta_fname = study_data_dir.join(format!("{}-meta.json", &common.StudyInstanceUID));
    if !study_meta_fname.is_file() {
        let study_meta_data = StudyDataMeta::new(&dcmtags, common);
        let data: HashMap<_, _> = [(&common.StudyInstanceUID, study_meta_data)].into();
        write_json(data, study_meta_fname)?;
    }

    // write stuff to seriesData/Y.Y.Y.YYYYY-meta.json
    let series_meta_fname =
        series_data_dir.join(format!("{}-meta.json", &common.SeriesInstanceUID));
    if !series_meta_fname.is_file() {
        let series_meta_data = SeriesDataMeta::new(&dcmtags, common);
        write_json(series_meta_data, series_meta_fname)?;
    }

    // write stuff to seriesData/Y.Y.Y.YYYYY-img/Z.Z.Z.ZZZZZ.dcm.json
    let img_data_dir = series_data_dir.join(format!("{}-img", &common.SeriesInstanceUID));
    fs_err::create_dir_all(&img_data_dir)?;
    let img_data_fname = img_data_dir.join(format!("{}.json", unpack.fname));
    let img_data = InstanceData::new(&dcmtags, common, &unpack.fname, unpack.path.as_str());
    let data: HashMap<_, _> = [(&common.SeriesInstanceUID, img_data)].into();
    write_json(data, img_data_fname)?;

    // write stuff to seriesData/Y.Y.Y.YYYYY-pack.json
    let pack_fname = series_data_dir.join(format!("{}-pack.json", &common.SeriesInstanceUID));
    if !pack_fname.is_file() {
        write_json(SERIES_PACK, pack_fname)?;
    }

    Ok(dcmtags.errors.into_inner())
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
