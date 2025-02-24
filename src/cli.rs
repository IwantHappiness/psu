use clap::{Parser, Subcommand};
use psu::Password;

#[derive(Debug, Parser)]
#[command(version,about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn run() -> Cli {
        Cli::parse()
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Add password")]
    Add {
        service: String,
        login: String,
        password: String,
    },

    #[clap(about = "Show password")]
    Print {
        #[arg(help = "ID of the password to display")]
        id: Option<u16>,

        #[arg(short, long, help = "Display all passwords", default_value_t = false)]
        all: bool,
    },

    #[clap(about = "Change password")]
    Modify(Password),

    #[clap(about = "Remove password")]
    Remove {
        #[arg(help = "ID")]
        id: Option<u16>,

        #[arg(short, long, default_value_t = false, help = "Remove all passwords")]
        all: bool,
    },
}
