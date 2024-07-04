use clap::{Parser, Subcommand};
use simplelog::{TermLogger, LevelFilter, Config, TerminalMode, ColorChoice};
use log::error;

extern crate exitcode;

mod ghrepo;
mod xbps;
mod settings;

use crate::ghrepo::{github_artifacts, github_update};
use crate::xbps::xbps_update_check;
use crate::settings::Settings;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Verbose operation, can be used multiple times to raise verbosity level
    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update the system
    Update,
    /// Check for the updates
    Checkupdate,
    /// Update custom packages from github actions
    Ghupdate {
        /// Select the artifact name to be used instead of the latest one
        #[arg(short, long)]
        artifact: Option<String>,

        /// List available artifatcs
        #[arg(short, long)]
        list: bool,
    },
}


fn main() {
    let ret = exitcode::OK;
    let cli = Cli::parse();

    Settings::init(cli.config);

    let verbosity = match cli.verbose {
        0 => Settings::verbosity(),
        x => x
    };

    let level = match verbosity {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    TermLogger::init(
        level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto
    ).unwrap();

    match &cli.command {
        Commands::Update => {
            error!("Update not implemented yet!");
        }
        Commands::Checkupdate => {
            xbps_update_check("/")
        }
        Commands::Ghupdate{ artifact, list } => {
            if *list {
                github_artifacts();
            } else {
                github_update("/", artifact.clone());
            }
        }
    }

    std::process::exit(ret)
}
