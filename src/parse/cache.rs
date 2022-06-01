// Backup https://channels.nixos.org/nixos-22.05/options.json.br

use std::{env, process::Command, path::Path, fs::{self, File}, io::{Write, Read, self, Cursor}, error::Error};
use curl::easy::Easy;
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
    let mut relver = version.split('.').collect::<Vec<&str>>()[0..2].join(".");
    
    let vout = Command::new("nix-instantiate")
        .arg("<nixpkgs/lib>")
        .arg("-A")
        .arg("version")
        .arg("--eval")
        .arg("--json")
        .output()?;

    let dlver = String::from_utf8_lossy(&vout.stdout)
        .to_string()
        .replace('"', "");

    if dlver.len() >= 8 && &dlver[5..8] == "pre" {
        relver = "unstable".to_string();
    }

    let cachedir = format!("{}/.cache/nixos-conf-editor", env::var("HOME")?);
    fs::create_dir_all(&cachedir).expect("Failed to create cache directory");
    let url = format!(
        "https://releases.nixos.org/nixos/{}/nixos-a{}/options.json.br",
        relver, dlver
    );

    let mut dst = Vec::new();
    let mut easy = Easy::new();
    easy.url(&url)?;

    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            })?;
        transfer.perform()?;
    }

    if easy.response_code()? == 404 {
        let url = format!("https://channels.nixos.org/nixos-{}/options.json.br", relver);
        easy = Easy::new();
        dst = Vec::new();
        easy.url(&url)?;
        easy.follow_location(true)?;
        {
            let mut transfer = easy.transfer();
            transfer
                .write_function(|data| {
                    dst.extend_from_slice(data);
                    Ok(data.len())
                })?;
            transfer.perform()?;
        }
        if easy.response_code()? == 404 {
            return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Failed to download options.json")));
        }
    }

    {
        let mut file = File::create(format!("{}/options.json", &cachedir).as_str())?;
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
    Ok(())
}