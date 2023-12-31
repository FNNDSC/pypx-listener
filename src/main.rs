use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Parser;
use rx_repack::{json_message, repack};

#[derive(clap::Parser)]
#[clap(
    about,
    long_about = r#"
px-repack is typically dispatched by storescp. Its purpose is to reoganize
the specified DICOM file to a path under the given data directory, putting
DICOM tag information into its new path.

The path template is:

 %PatientID-%PatientName-%PatientBirthDate
 └──%StudyDescription-%AccessionNumber-%StudyDate
    └──%_pad|5,0_SeriesNumber-%SeriesDescription-%_hash|%SeriesInstanceUID
       └──%_pad|4,0_InstanceNumber-%SOPInstanceUID.dcm
"#
)]
struct Cli {
    /// Parent directory of DICOM instance
    #[clap(long)]
    xcrdir: Utf8PathBuf,
    /// File name of DICOM instance
    #[clap(long)]
    xcrfile: Utf8PathBuf,

    /// Output directory for DICOM files
    #[clap(long)]
    datadir: Utf8PathBuf,

    /// Output directory for pypx DICOM tag data JSON files
    #[clap(long)]
    logdir: Option<Utf8PathBuf>,

    /// Remove DICOM file from source location
    #[clap(long, default_value_t = false)]
    cleanup: bool,

    /// Deprecated option
    #[clap(long)]
    verbosity: Option<u8>,

    /// Write to stdout the outcome JSON
    #[clap(short, long, default_value_t = false)]
    log_ndjson: bool,
}

fn main() -> anyhow::Result<()> {
    let args: Cli = Cli::parse();
    let dicom_file = args.xcrdir.join(&args.xcrfile);
    let outcome = repack(
        &dicom_file,
        &args.datadir,
        args.logdir.as_ref().map(|p| p.as_path()),
        args.cleanup,
    );

    if args.log_ndjson {
        // 12-factor app recommends writing to stdout (not stderr)
        // https://12factor.net/logs
        // NDJson is a best practice for logging:
        // http://ndjson.org/
        println!("{}", json_message(&dicom_file, &outcome)?);
    }

    outcome
        .with_context(|| format!("Failed to pack: {}", &dicom_file))
        .map(|_| ())
}
