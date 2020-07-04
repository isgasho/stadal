#[macro_use]
extern crate log;
extern crate fern;
extern crate chrono;
extern crate dirs;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use futures::executor::block_on;
use log::{error, info, warn};

use core_lib::app::Stadal;
use xi_rpc::RpcLoop;
use futures::io::Error;


fn create_log_directory(path_with_file: &Path) -> io::Result<()> {
    let log_dir = path_with_file.parent().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "Unable to get the parent of the following Path: {}, Your path should contain a file name",
            path_with_file.display(),
        ),
    ))?;
    fs::create_dir_all(log_dir)?;
    Ok(())
}

fn setup_logging(logging_path: Option<&Path>) -> Result<(), fern::InitError> {
    let level_filter = match std::env::var("XI_LOG") {
        Ok(level) => match level.to_lowercase().as_ref() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            _ => log::LevelFilter::Info,
        },
        // Default to info
        Err(_) => log::LevelFilter::Info,
    };

    let mut fern_dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message,
            ))
        })
        .level(level_filter)
        .chain(io::stderr());

    if let Some(logging_file_path) = logging_path {
        create_log_directory(logging_file_path)?;

        fern_dispatch = fern_dispatch.chain(fern::log_file(logging_file_path)?);
    };

    // Start fern
    fern_dispatch.apply()?;
    info!("Logging with fern is set up");

    // Log details of the logging_file_path result using fern/log
    // Either logging the path fern is outputting to or the error from obtaining the path
    match logging_path {
        Some(logging_file_path) => info!("Writing logs to: {}", logging_file_path.display()),
        None => warn!("No path was supplied for the log file. Not saving logs to disk, falling back to just stderr"),
    }
    Ok(())
}

fn get_logging_directory_path<P: AsRef<Path>>(directory: P) -> Result<PathBuf, io::Error> {
    match dirs::data_local_dir() {
        Some(mut log_dir) => {
            log_dir.push(directory);
            Ok(log_dir)
        }
        None => Err(
            io::Error::new(
                io::ErrorKind::NotFound,
                "No standard logging directory known for this platform",
            ))
    }
}

#[tokio::main]
async fn main() {
    let mut state = Stadal::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut rpc_looper = RpcLoop::new(stdout);

    let mut directory_path = get_logging_directory_path(PathBuf::from("stadal")).unwrap();
    directory_path.push(PathBuf::from("stadal.log"));

    if let Err(e) = setup_logging(Some(directory_path.as_path())) {
        eprintln!("[ERROR] setup_logging returned error, logging not enabled: {:?}", e);
    }

    match rpc_looper.mainloop(|| stdin.lock(), &mut state) {
        Ok(_) => (),
        Err(err) => {
            error!("exited with error:\n{:?}", err);
            process::exit(1);
        }
    }
}
