extern crate zip;

use std::io::prelude::*;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;

fn get_base_path() -> PathBuf {
    let mut path = env::home_dir().unwrap_or(env::temp_dir());
    path.push(".cache");
    path.push(format!("{}-{}", "aria2c-gui-rs", env!("CARGO_PKG_VERSION")));
    let mut tmp = PathBuf::new();
    path.clone_into(&mut tmp);
    fs::create_dir_all(tmp).err().map(|err| {
        println!("{:?}", err);
    });
    path
}

pub fn init_files(file: &str, bytes: &[u8]) -> Box<PathBuf> {
    let mut path = get_base_path();
    path.push(file);
    let path = path.as_path();
    if !path.exists() {
        let mut file = OpenOptions::new()
            .mode(0o766)
            .create_new(true)
            .write(true)
            .open(path)
            .unwrap();
        file.write_all(bytes).unwrap();
    }
    let mut ret = PathBuf::new();
    path.clone_into(&mut ret);
    return Box::new(ret);
}

pub fn unzip(dir: &str, bytes: &[u8]) -> PathBuf {
    let reader = ::std::io::Cursor::new(bytes);
    let mut path = get_base_path();

    path.push(dir);

    if path.exists() {
        return path;
    }

    let mut zip = zip::ZipArchive::new(reader).unwrap();

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        debug!("Filename: {}", file.name());
        let mut tmp = PathBuf::new();
        path.clone_into(&mut tmp);
        tmp.push(file.name());
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).err().map(|err| {
            println!("{:?}", err);
        });

        let fname = file.name();
        if &fname[fname.len() - 1..fname.len() - 0] == "/" {
            fs::create_dir_all(tmp).err().map(|err| {
                println!("{:?}", err);
            });
            continue;
        }

        fs::create_dir_all(tmp.parent().unwrap()).err().map(|err| {
            println!("{:?}", err);
        });
        let mut tmp2 = PathBuf::new();
        tmp.clone_into(&mut tmp2);
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(tmp)
            .map(|mut file| {
                file.write_all(bytes.as_slice()).err().map(|err| {
                    println!("{:?} {:?}", err, &file);
                });
                file
            })
            .err()
            .map(|err| {
                println!("{:?} {:?}", err, &tmp2);
            });
    }

    path
}
