mod util;

use anyhow::{anyhow, bail, Context, Error};
use std::{convert::TryFrom, path::PathBuf};
use structopt::StructOpt;
use tame_gcs::objects::{self, Metadata, Object};

#[derive(StructOpt, Debug)]
pub(crate) struct Args {
    /// A gs: URL or filepath for the source path to copy from,
    /// wildcards are not currently supported
    src_url: String,
    /// A gs: URL or filepath for the destination to copy to,
    /// wildcards are not currently supported
    dest_url: String,
}

enum DataPath {
    Gs(util::GsUrl),
    File(PathBuf),
}

impl DataPath {
    fn is_file(&self) -> bool {
        if let DataPath::File(_) = self {
            true
        } else {
            false
        }
    }
}

impl TryFrom<String> for DataPath {
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.starts_with("gs://") {
            let url = url::Url::parse(&s)?;
            Ok(DataPath::Gs(util::gs_url_to_object_id(&url)?))
        } else {
            Ok(DataPath::File(PathBuf::from(s)))
        }
    }
}

// cp is probably gsutil's most complicated subcommand, so we only implement
// a bare minimum
pub(crate) fn cmd(ctx: &util::RequestContext, args: Args) -> Result<(), Error> {
    use std::fs;

    let src = DataPath::try_from(args.src_url)?;
    let dst = DataPath::try_from(args.dest_url)?;

    // Just support gcs to local or vice versa, not local to local or gcs to gcs
    if src.is_file() == dst.is_file() {
        let location = if src.is_file() { "local disk" } else { "gcs" };

        bail!("source and destination are both located on {}", location)
    }

    match (&src, &dst) {
        (DataPath::File(ref src), DataPath::Gs(dst)) => {
            let src_file = fs::File::open(src).context("source path")?;
            let src_len = src_file.metadata()?.len();

            let obj_name = format!(
                "{}{}{}",
                dst.object().map(|on| on.as_ref()).unwrap_or(""),
                if dst.object().is_some() { "/" } else { "" },
                src.file_name()
                    .as_ref()
                    .and_then(|os| os.to_str())
                    .ok_or_else(|| anyhow!("can't turn file_name into string"))?
            );
            let insert_req = Object::insert_multipart(
                dst.bucket(),
                src_file,
                src_len,
                &Metadata {
                    name: Some(obj_name),
                    ..Default::default()
                },
                None,
            )?;

            let _insert_res: objects::InsertResponse = util::execute(ctx, insert_req)?;

            Ok(())
        }
        (DataPath::Gs(src), DataPath::File(dst)) => {
            let mut dst_file = fs::File::create(dst).context("destination path")?;

            let dl_req = Object::download(
                &(
                    src.bucket(),
                    src.object()
                        .ok_or_else(|| anyhow!("must provide a full object name to copy from"))?,
                ),
                None,
            )?;

            let mut response: objects::DownloadObjectResponse = util::execute(ctx, dl_req)?;

            std::io::copy(&mut response, &mut dst_file)?;

            Ok(())
        }
        _ => unreachable!(),
    }
}

#[derive(StructOpt)]
enum Command {
    /// Copy files and objects
    #[structopt(name = "cp")]
    Cp(Args),
}

#[derive(StructOpt)]
#[structopt(name = "gsutil")]
struct Opts {
    #[structopt(short, long, parse(from_os_str))]
    credentials: Option<PathBuf>,
    #[structopt(subcommand)]
    cmd: Command,
}

fn real_main() -> Result<(), Error> {
    let args = Opts::from_args();

    let cred_path = args
        .credentials
        .or_else(|| std::env::var_os("GOOGLE_APPLICATION_CREDENTIALS").map(PathBuf::from))
        .ok_or_else(|| anyhow::anyhow!("missing GOOGLE_APPLICATION_CREDENTIALS env var"))?;

    let client = reqwest::Client::builder().build()?;
    let svc_account_info =
        tame_oauth::gcp::ServiceAccountInfo::deserialize(std::fs::read_to_string(&cred_path)?)?;
    let svc_account_access = tame_oauth::gcp::ServiceAccountAccess::new(svc_account_info)?;

    let ctx = util::RequestContext {
        client,
        cred_path,
        auth: std::sync::Arc::new(svc_account_access),
    };

    match args.cmd {
        Command::Cp(args) => cmd(&ctx, args),
    }
}

fn main() {
    match real_main() {
        Ok(_) => {}
        Err(e) => {
            println!("{}", format!("{}", e));
            std::process::exit(1);
        }
    }
}
