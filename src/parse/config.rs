use nix_editor;
use std::{fs, path::Path, collections::HashMap, error::Error, io};

pub fn parseconfig(path: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let f = fs::read_to_string(Path::new(path))?;
    match nix_editor::parse::get_collection(f) {
        Ok(x) => Ok(x),
        Err(_) => Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Failed to parse config"))),
    }
}

pub fn opconfigured<T>(conf: &HashMap<String, T>, pos: &[String], attr: String) -> bool {
    let mut p = pos.to_vec();
    p.push(attr);
    conf.keys().any(|k| {
        let s = k.split('.').collect::<Vec<_>>();
        if s.len() < p.len() {
            false
        } else {
            s[0..p.len()].eq(&p)
        }
    })
}

pub fn editconfig(path: &str, editedopts: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
    let mut f = fs::read_to_string(Path::new(path))?;
    for (op, val) in editedopts.into_iter() {
        if val.is_empty() {
            f = match nix_editor::write::deref(&f, &op) {
                Ok(x) => x,
                Err(_) => return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Failed to deref {}", op)))),
            };
        } else {
            f = match nix_editor::write::write(&f, &op, &val) {
                Ok(x) => x,
                Err(_) => return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Failed to set value {} to {}", op, val)))),
            };
        }
    }
    Ok(f)
}