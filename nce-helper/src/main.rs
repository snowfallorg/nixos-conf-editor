use clap::{self, FromArgMatches, Subcommand};
use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    process::Command,
};

#[derive(clap::Subcommand)]
enum SubCommands {
    Config {
        /// Write stdin to file in path output
        #[clap(short, long)]
        output: String,
    },
    Rebuild {
        /// Run `nixos-rebuild` with the given arguments
        arguments: Vec<String>,
    },
}

fn main() {
    let cli = SubCommands::augment_subcommands(clap::Command::new("Helper binary for NixOS Configuration Editor"));
    let matches = cli.get_matches();
    let derived_subcommands = SubCommands::from_arg_matches(&matches)
        .map_err(|err| err.exit())
        .unwrap();

    if users::get_effective_uid() != 0 {
        eprintln!("nixos-conf-editor-helper must be run as root");
        std::process::exit(1);
    }

    match derived_subcommands {
        SubCommands::Config { output } => {
            match write_file(&output) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            };
        }
        SubCommands::Rebuild { arguments } => match rebuild(arguments) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        },
    }
}

fn write_file(path: &str) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut buf = String::new();
    stdin.lock().read_to_string(&mut buf)?;
    let mut file = File::create(path)?;
    write!(file, "{}", &buf)?;
    Ok(())
}

fn rebuild(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new("nixos-rebuild").args(args).spawn()?;
    let x = cmd.wait()?;
    if x.success() {
        Ok(())
    } else {
        eprintln!("nixos-rebuild failed with exit code {}", x.code().unwrap());
        std::process::exit(1);
    }
}
