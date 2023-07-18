use camino::Utf8PathBuf;
use clap::Parser;
use rx_repack::repack;

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
    └──%_pad|5,0_SeriesNumber-%SeriesDescription
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

    /// output directory
    #[clap(long)]
    datadir: Utf8PathBuf,

    /// Remove DICOM file from source location
    #[clap(long, default_value_t = false)]
    cleanup: bool,

    /// NOT IMPLEMENTED
    #[clap(long)]
    logdir: Option<Utf8PathBuf>,

    /// Deprecated option
    #[clap(long)]
    verbosity: Option<u8>,
}

fn main() -> anyhow::Result<()> {
    let args: Cli = Cli::parse();
    let dicom_file = args.xcrdir.join(&args.xcrfile);
    repack(
        &dicom_file,
        &args.datadir,
        args.logdir.as_ref().map(|p| p.as_path()),
        args.cleanup,
    )
}
