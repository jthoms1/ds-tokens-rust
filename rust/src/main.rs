mod lib;
use lib::{css_variables, json, scss_variables, typescript, FlatTokenList};
use rayon::prelude::*;
use serde_json::Value;
use std::ffi::OsStr;
use std::fmt;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::ErrorKind;
use std::io::Write;
use std::str::FromStr;
use std::string::ToString;
use structopt::StructOpt;

#[derive(Debug)]
enum Transform {
    CSS,
    SCSS,
    JSON,
    TS,
}
impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
impl FromStr for Transform {
    type Err = ();
    fn from_str(input: &str) -> Result<Transform, Self::Err> {
        match input {
            "css" => Ok(Transform::CSS),
            "ts" => Ok(Transform::TS),
            "json" => Ok(Transform::JSON),
            "scss" => Ok(Transform::SCSS),
            _ => Err(()),
        }
    }
}

#[derive(StructOpt)]
struct Cli {
    file_path: std::path::PathBuf,
    out_dir: std::path::PathBuf,
}

#[derive(Debug)]
struct CustomError(String);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Cli::from_args().file_path;
    let out_dir = Cli::from_args().out_dir;
    let file_types = ["css", "json", "ts", "scss"];
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

    let flat_token_list = convert_to_flat_list(&contents, vec![], vec![]);

    file_types
        .par_iter()
        .map(|file_type| {
            let results = match Transform::from_str(&file_type).unwrap() {
                Transform::JSON => json(&contents).unwrap(),
                Transform::TS => typescript(&contents).unwrap(),
                Transform::CSS => css_variables(&flat_token_list).unwrap(),
                Transform::SCSS => scss_variables(&flat_token_list).unwrap(),
            };
            let file_results = File::create(
                output_directory
                    .join(file_stem)
                    .with_extension(file_type.to_string()),
            );
            match file_results {
                Ok(mut json_file) => json_file.write_all(results.as_bytes()),
                Err(e) => panic!("Problem creating the file {:?}", e),
            }
        })
        .count();

    Ok(())
}

fn convert_to_flat_list(
    value: &Value,
    value_list: FlatTokenList,
    prefix_list: Vec<String>,
) -> FlatTokenList {
    let mut new_value_list = value_list.to_vec();

    let mut new_token_values: FlatTokenList = match value {
        Value::Null => Vec::new(),
        Value::String(ref str) => vec![(prefix_list, str.to_string())],
        Value::Number(ref num) => vec![(prefix_list, num.to_string())],
        Value::Array(ref array_values) => array_values
            .iter()
            .enumerate()
            .flat_map(|(index, x)| {
                let mut newvec = prefix_list.to_vec();
                newvec.push((index + 1).to_string());
                return convert_to_flat_list(x, value_list.to_vec(), newvec);
            })
            .collect(),
        Value::Object(ref object) => object
            .keys()
            .flat_map(|key| {
                let mut newvec = prefix_list.to_vec();
                newvec.push(key.to_string());
                convert_to_flat_list(&object[key], value_list.to_vec(), newvec)
            })
            .collect(),
        _ => vec![(prefix_list, value.to_string())],
    };

    new_value_list.append(&mut new_token_values);

    return new_value_list;
}
