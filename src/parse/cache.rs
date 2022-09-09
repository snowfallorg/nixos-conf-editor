use std::{env, process::Command, path::Path, fs::{self, File}, io::{Write, Read, self}, error::Error};
use log::*;
use serde_json::Value;


pub fn checkcache() -> Result<(), Box<dyn Error>> {
    let cachedir = format!("{}/.cache/nixos-conf-editor", env::var("HOME")?);

    let vout = Command::new("nixos-version")
        .arg("--json")
        .output()?;
    let data: Value =
        serde_json::from_str(&String::from_utf8_lossy(&vout.stdout))?;

    let vererr = Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Failed to find version")));
    let version = match data.as_object() {
        Some(x) => match x["nixosVersion"].as_str() {
            Some(y) => y,
            None => return vererr.unwrap(),
        },
        None => return vererr.unwrap(),
    };
    debug!("NixOS version: {}", version);

    if !Path::is_dir(Path::new(&cachedir))
        || !Path::is_file(Path::new(&format!("{}/version.json", &cachedir)))
    {
        setupcache(version)?;
        let mut newver = fs::File::create(format!("{}/version.json", &cachedir))?;
        newver.write_all(&vout.stdout)?;
    }

    let file = fs::read_to_string(format!("{}/version.json", cachedir))?;
    let olddata: Value = serde_json::from_str(&file)?;

    let oldversion = match olddata.as_object() {
        Some(x) => match x["nixosVersion"].as_str() {
            Some(y) => y,
            None => return vererr.unwrap(),
        },
        None => return vererr.unwrap(),
    };

    if version != oldversion || !Path::is_file(Path::new(&format!("{}/options.json", &cachedir))){
        setupcache(version)?;
        let mut newver = fs::File::create(format!("{}/version.json", &cachedir))?;
        newver.write_all(&vout.stdout)?;
    }
    Ok(())
}

fn setupcache(version: &str) -> Result<(), Box<dyn Error>> {
    info!("Setting up cache");

    let mut relver = version[0..5].to_string();
    if relver == "22.11" {
        relver = "unstable".to_string();
    }

    let cachedir = format!("{}/.cache/nixos-conf-editor", env::var("HOME")?);
    fs::create_dir_all(&cachedir).expect("Failed to create cache directory");
    let url = format!(
        "https://channels.nixos.org/nixos-{}/options.json.br",
        relver
    );

    dlfile(&url, &format!("{}/options.json", &cachedir))?;
    Ok(())
}

fn dlfile(url: &str, path: &str) -> Result<(), Box<dyn Error>> {
    trace!("Downloading {}", url);
    let response = reqwest::blocking::get(url)?;
    if response.status().is_success() {
        let cachedir = format!("{}/.cache/nixos-conf-editor", env::var("HOME")?);
        if !Path::new(&cachedir).exists() {
            fs::create_dir_all(&cachedir).expect("Failed to create cache directory");
        }

        let dst: Vec<u8> = response.bytes()?.to_vec();
        {
            let mut file = File::create(path)?;
            let mut reader = brotli::Decompressor::new(
                dst.as_slice(),
                4096, // buffer size
            );
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf[..]) {
                    Err(e) => {
                        if let std::io::ErrorKind::Interrupted = e.kind() {
                            continue;
                        }
                        return Err(Box::new(e));
                    }
                    Ok(size) => {
                        if size == 0 {
                            break;
                        }
                        file.write_all(&buf[..size])?
                    }
                }
            }
        }
    }
    Ok(())
}
