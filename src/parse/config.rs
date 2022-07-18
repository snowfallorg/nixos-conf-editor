use log::{debug, info, trace, warn};
use nix_editor;
use std::{collections::HashMap, error::Error, fs, hash::Hash, io, path::Path};

pub fn parseconfig(path: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let f = fs::read_to_string(Path::new(path))?;
    match nix_editor::parse::get_collection(f) {
        Ok(x) => Ok(x),
        Err(_) => Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "Failed to parse config",
        ))),
    }
}

pub fn opconfigured<T: std::fmt::Debug>(
    conf: &HashMap<String, T>,
    pos: &[String],
    attr: String,
) -> bool {
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

pub fn opconfigured2(path: &str, pos: &[String], refpos: &[String], attr: String) -> bool {
    let mut p = pos.to_vec();
    p.push(attr.clone());
    let mut r = refpos.to_vec();
    r.push(attr);
    readval(path, &p.join("."), &r.join(".")).is_ok()
}

pub fn getconfvals<T>(conf: &HashMap<String, T>, pos: &[String]) -> Vec<String> {
    let mut out = vec![];
    for attr in conf.keys() {
        let k = attr.split('.').collect::<Vec<_>>();
        if k.len() > pos.len() && k[0..pos.len()].eq(pos) {
            let x = k[pos.len()].to_string();
            if !out.contains(&x) {
                out.push(x);
            }
        }
    }
    out
}

pub fn getarrvals(path: &str, pos: &[String]) -> Vec<String> {
    let f = fs::read_to_string(Path::new(path)).unwrap();
    let out = nix_editor::read::getarrvals(&f, &pos.join("."));
    match out {
        Ok(x) => x,
        Err(_) => vec![],
    }
}

pub fn editconfigpath(
    path: &str,
    editedopts: HashMap<String, String>,
) -> Result<String, Box<dyn Error>> {
    let f = fs::read_to_string(Path::new(path))?;
    editconfig(f, editedopts)
}

pub fn editconfig(
    mut f: String,
    //path: &str,
    editedopts: HashMap<String, String>,
) -> Result<String, Box<dyn Error>> {
    debug!("editedopts: {:#?}", editedopts);
    let mut k = editedopts.keys().collect::<Vec<_>>();
    k.sort();

    let mut starops: HashMap<String, HashMap<usize, String>> = HashMap::new();
    for (op, val) in editedopts.into_iter() {
        if op.split('.').any(|x| x.parse::<usize>().is_ok()) {
            let option = op.split('.').collect::<Vec<_>>();
            let index = option
                .iter()
                .position(|x| x.parse::<usize>().is_ok())
                .unwrap();
            let o = &option[..index];
            let v = &option[index + 1..];
            let i = option[index].parse::<usize>().unwrap();

            let mut p = if let Some(y) = starops.get(&o.join(".")) {
                y.to_owned()
            } else {
                HashMap::new()
            };
            // fill up on first time
            if p.is_empty() {
                let arr = match nix_editor::read::getarrvals(&f, &o.join(".")) {
                    Ok(x) => x,
                    Err(_) => vec![],
                };
                for j in 0..arr.len() {
                    p.insert(j, arr[j].to_string());
                }
            }

            let arrval = match p.get(&i) {
                Some(x) => x.to_string(),
                None => "{}".to_string(),
            };
            let mut h = HashMap::new();
            h.insert(v.join("."), val);
            p.insert(i, editconfig(arrval, h)?);
            starops.insert(o.join("."), p);
        } else if val.is_empty() {
            f = match nix_editor::write::deref(&f, &op) {
                Ok(x) => x,
                Err(_) => {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to deref {}", op),
                    )))
                }
            };
        } else {
            f = match nix_editor::write::write(&f, &op, &val) {
                Ok(x) => x,
                Err(_) => {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to set value {} to {}", op, val),
                    )))
                }
            };
        }
    }
    for (k, v) in starops {
        let mut arr = v.into_iter().collect::<Vec<_>>();
        arr.sort_by(|(x, _), (y, _)| x.cmp(y));
        // Just use nixpkgs-fmt instead
        let valarr = format!(
            "[\n{}\n  ]",
            arr.iter()
                .filter(|(_, x)| x.trim().replace('\n', "").replace(' ', "") != "{}")
                .map(|(_, y)| format!(
                    "  {}",
                    y.replace(";\n  ", ";\n    ")
                        .replace("; }", ";\n  }")
                        .replace("; ", ";\n    ")
                        .replace(";\n      ", ";\n    ")
                        .replace("{ ", "{\n    ")
                        .replace(";}", ";\n  }")
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );
        f = match nix_editor::write::write(&f, &k, &valarr) {
            Ok(x) => x,
            Err(_) => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to set value {} to {}", k, valarr),
                )))
            }
        };
    }
    Ok(f)
}

pub fn readval(path: &str, query: &str, refq: &str) -> Result<String, Box<dyn Error>> {
    warn!("READVAL: {} {} {}", path, query, refq);
    let f = fs::read_to_string(Path::new(path))?;
    let out = if !refq.contains(&String::from("*")) {
        nix_editor::read::readvalue(&f, query)
    } else {
        let p = refq.split('.').collect::<Vec<_>>();
        let mut r: Vec<Vec<String>> = vec![vec![]];
        let mut indexvec: Vec<usize> = vec![];
        let mut j = 0;
        for i in 0..p.len() {
            if p[i] == "*" {
                r.push(vec![]);
                if let Ok(x) = query.split('.').collect::<Vec<_>>()[i].parse::<usize>() {
                    indexvec.push(x);
                }
                j += 1;
            } else {
                r[j].push(p[i].to_string());
            }
        }
        let mut f = fs::read_to_string(Path::new(path)).unwrap();
        let mut i = 0;
        for y in r {
            if i < indexvec.len() {
                f = match nix_editor::read::getarrvals(&f, &y.join(".")) {
                    Ok(x) => {
                        warn!("x: {:?}", x);
                        let o = match x.get(indexvec[i]) {
                            Some(x) => x.to_string(),
                            None => {
                                return Err(Box::new(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("Index out of bounds {}", refq),
                                )))
                            }
                        };
                        i += 1;
                        o
                    }
                    Err(_) => {
                        return Err(Box::new(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Failed to read value from configuration",
                        )))
                    }
                };
            } else {
                f = match nix_editor::read::readvalue(&f, &y.join(".")) {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(Box::new(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Failed to read value from configuration",
                        )))
                    }
                };
            }
        }
        Ok(f)
    };
    match out {
        Ok(x) => Ok(x),
        Err(_) => Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to read value {}", query),
        ))),
    }
}
