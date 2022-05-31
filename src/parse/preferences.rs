use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path, error::Error,
};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    systemconfig: String,
    flake: Option<String>,
}

pub fn checkconfig() -> Result<String, Box<dyn Error>> {
    let cfgdir = format!("{}/.config/nixos-conf-editor", env::var("HOME")?);
    if !Path::is_file(Path::new(&format!("{}/config.json", &cfgdir))) {
        if !Path::is_file(Path::new("/etc/nixos-conf-editor/config.json")) {
            createdefaultconfig()?;
            Ok(cfgdir)
        } else {
            Ok("/etc/nixos-conf-editor/".to_string())
        }
    } else {
        Ok(cfgdir)
    }
}

pub fn configexists() -> Result<bool, Box<dyn Error>> {
    let cfgdir = format!("{}/.config/nixos-conf-editor", env::var("HOME")?);
    if !Path::is_file(Path::new(&format!("{}/config.json", &cfgdir))) {
        if !Path::is_file(Path::new("/etc/nixos-conf-editor/config.json")) {
            Ok(false)
        } else {
            Ok(true)
        }
    } else {
        Ok(true)
    }
}

pub fn createconfig(systemconfig: String, flake: Option<String>) -> Result<(), Box<dyn Error>> {
    let cfgdir = format!("{}/.config/nixos-conf-editor", env::var("HOME")?);
    fs::create_dir_all(&cfgdir)?;
    let config = Config {
        systemconfig,
        flake,
    };
    let json = serde_json::to_string_pretty(&config)?;
    let mut file = File::create(format!("{}/config.json", cfgdir))?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn createdefaultconfig() -> Result<(), Box<dyn Error>> {
    let cfgdir = format!("{}/.config/nixos-conf-editor", env::var("HOME")?);
    fs::create_dir_all(&cfgdir)?;
    let config = Config {
        systemconfig: "/etc/nixos/configuration.nix".to_string(),
        flake: None,
    };
    let json = serde_json::to_string_pretty(&config)?;
    let mut file = File::create(format!("{}/config.json", cfgdir))?;
    file.write_all(json.as_bytes())?;
    Ok(())
}


pub fn readconfig(cfg: String) -> Result<(String, Option<String>), Box<dyn Error>> {
    let file = fs::read_to_string(cfg)?;
    let config: Config = match serde_json::from_str(&file) {
        Ok(x) => x,
        Err(_) => {
            createdefaultconfig()?;
            return Ok((
                "/etc/nixos/configuration.nix".to_string(),
                None,
            ));
        }
    };
    if Path::is_file(Path::new(&config.systemconfig)) {
        Ok((config.systemconfig, config.flake))
    } else {
        Ok((
            "/etc/nixos/configuration.nix".to_string(),
            None,
        ))
    }
}
