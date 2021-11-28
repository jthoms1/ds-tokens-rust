mod lib;
use lib::{Process, Transform};
use rayon::prelude::*;
use serde_json::Value;
use std::ffi::OsStr;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::ErrorKind;
use std::io::Write;
use std::str::FromStr;
use std::string::ToString;
use structopt::StructOpt;
use strum::VariantNames;

#[cfg(test)]
mod tests;

#[derive(StructOpt)]
struct Cli {
    file_path: std::path::PathBuf,
    out_dir: std::path::PathBuf,
    file_types: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Cli::from_args().file_path;
    let out_dir = Cli::from_args().out_dir;

    let file_types: Vec<String> = match Cli::from_args().file_types.is_empty() {
        true => Transform::VARIANTS
            .to_vec()
            .into_iter()
            .map(|str| str.to_owned())
            .collect(),
        false => Cli::from_args().file_types,
    };

    let output_directory = &out_dir;
    let file_stem = &path.file_stem().unwrap();
    let extension = &path.extension().and_then(OsStr::to_str);
    let content = read_to_string(&path)?;

    // Parse the contents and use the appropriate serializer from file type
    let contents: Value = match extension {
        Some("json") => serde_json::from_str(&content)?,
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
        _ => panic!(
            "'{:?}' is not a supported extension type.  Please us a yaml or json file.",
            extension
        ),
    };

    // Create the output directory path based on the prop provided
    create_dir_all(output_directory).unwrap_or_else(|error| {
        if error.kind() != ErrorKind::AlreadyExists {
            panic!("Problem creating the directory {:?}", error)
        }
    });

    file_types
        .par_iter()
        .filter_map(|file_type| match Transform::from_str(file_type) {
            Ok(file_type) => Some(file_type),
            Err(_) => {
                println!("Transform type {} is not supported", file_type);
                None
            }
        })
        .map(|transform_type| {
            let results = transform_type.process(&contents).unwrap();
            let file_results = File::create(
                output_directory
                    .join(file_stem)
                    .with_extension(transform_type.to_string()),
            );
            match file_results {
                Ok(mut file) => file.write_all(results.as_bytes()),
                Err(e) => panic!("Problem creating the file {:?}", e),
            }
        })
        .count();

    Ok(())
}
