// Backup https://channels.nixos.org/nixos-22.05/options.json.br

use std::{env, process::Command, path::Path, fs::{self, File}, io::{Write, Read}, error::Error};
use curl::easy::Easy;
use serde_json::Value;


pub fn checkcache() -> Result<(), Box<dyn Error>> {
    let cachedir = format!("{}/.cache/nixos-conf-editor", env::var("HOME")?);

    let vout = Command::new("nixos-version")
        .arg("--json")
        .output()
        .unwrap();
    let data: Value =
        serde_json::from_str(&String::from_utf8_lossy(&vout.stdout))?;

    let version = data.as_object().unwrap()["nixosVersion"].as_str().unwrap();

    if !Path::is_dir(Path::new(&cachedir))
        || !Path::is_file(Path::new(&format!("{}/version.json", &cachedir)))
    {
        setupcache(version)?;
        let mut newver = fs::File::create(format!("{}/version.json", &cachedir))?;
        newver.write_all(&vout.stdout)?;
    }

    let file = fs::read_to_string(format!("{}/version.json", cachedir))?;
    let olddata: Value = serde_json::from_str(&file)?;

    let oldversion = olddata.as_object().unwrap()["nixosVersion"]
        .as_str()
        .unwrap();

    if version != oldversion || !Path::is_file(Path::new(&format!("{}/options.json", &cachedir))){
        setupcache(version)?;
        let mut newver = fs::File::create(format!("{}/version.json", &cachedir))?;
        newver.write_all(&vout.stdout)?;
    }
    Ok(())
}

fn setupcache(version: &str) -> Result<(), Box<dyn Error>> {
    let mut relver = version.split('.').collect::<Vec<&str>>()[0..2].join(".");
    if &relver[0..5] == "22.11" {
        relver = "unstable".to_string();
    }

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

    let cachedir = format!("{}/.cache/nixos-conf-editor", env::var("HOME")?);
    fs::create_dir_all(&cachedir).expect("Failed to create cache directory");
    let url = format!(
        "https://releases.nixos.org/nixos/{}/nixos-{}/options.json.br",
        relver, dlver
    );

    /*if Path::is_file(Path::new(&format!("{}/options.json", &cachedir))) {
        fs::remove_file(Path::new(&format!("{}/options.json", &cachedir)))
            .expect("Failed to remove file");
    }*/

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
        transfer.perform()?
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
                    panic!("{}", e);
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