use std::fs::{File, read_to_string, create_dir_all};
use std::io::Write;
use std::ffi::OsStr;
use std::path::Path;
use std::io::ErrorKind;
use structopt::StructOpt;
use serde_json::Value;

#[derive(StructOpt)]
struct Cli {
    file_path: std::path::PathBuf,
    out_dir: std::path::PathBuf
}

#[derive(Debug)]
struct CustomError(String);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Cli::from_args().file_path;
    let out_dir = Cli::from_args().out_dir;

    let output_directory = Path::new(&out_dir);
    let file_stem = Path::new(&path).file_stem().unwrap();
    let extension = Path::new(&path).extension().and_then(OsStr::to_str);
    let content = read_to_string(&path)?;

    // Parse the contents and use the appropriate serializer from file type
    let contents: Value = match extension {
        Some("json") => serde_json::from_str(&content)?,
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
        _ => panic!("'{:?}' is not a supported extension type.  Please us a yaml or json file.", extension)
    };

    // Create the output directory path based on the prop provided
    create_dir_all(output_directory).unwrap_or_else(|error| {
        if error.kind() != ErrorKind::AlreadyExists {
            panic!("Problem opening the file: {:?}", error)
        }
    });

    // Create JSON output
    let json_contents = transform_to_json(&contents).unwrap();
    let mut json_file = File::create(output_directory.join(file_stem).with_extension(json_contents.0))?;
    json_file.write_all(json_contents.1.as_bytes())?;

    // Create TS output
    let ts_contents = transform_to_typescript(&contents).unwrap();
    let mut ts_file = File::create(output_directory.join(file_stem).with_extension(ts_contents.0))?;
    ts_file.write_all(ts_contents.1.as_bytes())?;

    Ok(())
}

fn transform_to_json(contents: &Value) -> Result<(String, String), serde_json::Error> {
    let results = match serde_json::to_string(&contents) {
        Ok(results) => results,
        Err(e) => return Err(e)
    };

    return Ok(("json".to_string(), results));
}

fn transform_to_typescript(contents: &Value) -> Result<(String, String), serde_json::Error> {
    let results = match serde_json::to_string(&contents) {
        Ok(results) => results,
        Err(e) => return Err(e)
    };

    let output = format!("
  export const themeData = {} as const;\n
  export type ThemeType = typeof themeData;
  ", results);

  
    return Ok(("ts".to_string(), output));
}

/*
fn transformToCSSVariables() -> Result<String>{

}

fn transformToSCSSVariables() -> Result<String>{

}
*/