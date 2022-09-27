use std::{
    env,
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct NceConfig {
    pub systemconfig: String,
    pub flake: Option<String>,
    pub flakearg: Option<String>,
}

impl Default for NceConfig {
    fn default() -> NceConfig {
        NceConfig {
            systemconfig: String::from("/etc/nixos/configuration.nix"),
            flake: None,
            flakearg: None,
        }
   }
}

pub fn getconfig() -> Option<NceConfig> {
    if let Ok(c) = getconfigval() {
        Some(c)
    } else {
        None
    }
}

fn getconfigval() -> Result<NceConfig, Box<dyn Error>> {
    let configfile = checkconfig()?;
    let config: NceConfig =
        serde_json::from_reader(File::open(format!("{}/config.json", configfile))?)?;
    Ok(config)
}

fn checkconfig() -> Result<String, Box<dyn Error>> {
    let cfgdir = format!("{}/.config/nixos-conf-editor", env::var("HOME")?);
    if !Path::is_file(Path::new(&format!("{}/config.json", &cfgdir))) {
        if !Path::is_file(Path::new("/etc/nixos-conf-editor/config.json")) {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No config file found",
            )))
        } else {
            Ok("/etc/nixos-conf-editor/".to_string())
        }
    } else {
        Ok(cfgdir)
    }
}

pub fn editconfig(config: NceConfig) -> Result<(), Box<dyn Error>> {
    let cfgdir = format!("{}/.config/nixos-conf-editor", env::var("HOME")?);
    fs::create_dir_all(&cfgdir)?;
    let json = serde_json::to_string_pretty(&config)?;
    let mut file = File::create(format!("{}/config.json", cfgdir))?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
