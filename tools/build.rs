use zip::read::ZipFile;
use zip::ZipArchive;

use std::env;
use std::fs::{create_dir, File};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;

const DIRECTORY: u32 = 2 << 13;
const FILE: u32 = 2 << 14;

fn main() {
    let protoc_zip_path = if cfg!(windows) {
        env::current_dir().unwrap().join("protoc-3.6.1-win32.zip")
    } else {
        panic!("TODO: support other OSs")
    };

    let out_dir = env::current_dir().unwrap().join("out");

    create_dir(&out_dir);

    println!("Reading {:?}", protoc_zip_path);

    unzip(&protoc_zip_path, &out_dir.join("protoc"));
}

fn unzip(zip_path: &Path, destination: &Path) {
    let zip_file = File::open(zip_path).expect("Failed to read zip file.");
    let reader = BufReader::new(zip_file);
    let mut archive = ZipArchive::new(reader).expect("Failed to read zip file.");

    create_dir(&destination);

    println!("{:?} {:?}", FILE, DIRECTORY);

    println!("cargo:rerun-if-changed={}", destination.to_string_lossy());

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();

        println!("{}, {:?}", file.name(), is_dir(&file));

        let file_destination = destination.join(file.name());

        if is_dir(&file) {
            println!("Creating directory {:?}", &file_destination);
            create_dir(&file_destination);
        } else if is_file(&file) {
            println!("Extracting {:?}", &file_destination);
            let out_file = File::create(&file_destination)
                .expect(&format!("Failed to create {:?}", file_destination));
            let mut writer = BufWriter::new(out_file);
            let mut data: Vec<u8> = vec![];

            file.read_to_end(&mut data);

            // TODO: set real permissions on Unix.

            writer.write(&data);
        }
    }
}

fn is_dir(file: &ZipFile) -> bool {
    file.unix_mode().unwrap() & DIRECTORY != 0
}

fn is_file(file: &ZipFile) -> bool {
    file.unix_mode().unwrap() & FILE != 0
}
