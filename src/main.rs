use convert_case::{Case, Casing};
use serde_json::Value;
use std::ffi::OsStr;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    file_path: std::path::PathBuf,
    out_dir: std::path::PathBuf,
}

#[derive(Debug)]
struct CustomError(String);

type FlatTokenList = Vec<(Vec<String>, String)>;

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

    // Create JSON output
    let json_contents = transform_to_json(&contents).unwrap();
    let mut json_file = File::create(
        output_directory
            .join(file_stem)
            .with_extension(json_contents.0),
    )?;
    json_file.write_all(json_contents.1.as_bytes())?;

    // Create TS output
    let ts_contents = transform_to_typescript(&contents).unwrap();
    let mut ts_file = File::create(
        output_directory
            .join(file_stem)
            .with_extension(ts_contents.0),
    )?;
    ts_file.write_all(ts_contents.1.as_bytes())?;

    // Create CSS Variable output
    let css_contents = transform_to_css_variables(&contents).unwrap();
    let mut css_file = File::create(
        output_directory
            .join(file_stem)
            .with_extension(css_contents.0),
    )?;
    css_file.write_all(css_contents.1.as_bytes())?;

    // Create SCSS Variable output
    let scss_contents = transform_to_scss_variables(&contents).unwrap();
    let mut scss_file = File::create(
        output_directory
            .join(file_stem)
            .with_extension(scss_contents.0),
    )?;
    scss_file.write_all(scss_contents.1.as_bytes())?;

    Ok(())
}

fn transform_to_json(contents: &Value) -> Result<(String, String), serde_json::Error> {
    let output = format!("{:#}", contents);

    return Ok(("json".to_string(), output));
}

fn transform_to_typescript(contents: &Value) -> Result<(String, String), serde_json::Error> {
    let output = format!(
        "
export const themeData = {:#} as const;\n
export type ThemeType = typeof themeData;
  ",
        contents
    );
    return Ok(("ts".to_string(), output));
}

fn transform_to_css_variables(contents: &Value) -> Result<(String, String), serde_json::Error> {
    let flat_token_list = convert_to_flat_list(&contents, vec![], vec![]);
    let results = flat_token_list
        .iter()
        .map(|(prefix_list, value)| {
            let css_varname = prefix_list
                .iter()
                .map(|key_name| key_name.to_case(Case::Kebab))
                .collect::<Vec<String>>()
                .join("-");
            return format!("  --{}: {};", css_varname, value);
        })
        .collect::<Vec<String>>()
        .join("\n");

    let output = format!(
        "
:root {{ 
{}
}}
",
        results
    );

    return Ok(("css".to_string(), output));
}

fn transform_to_scss_variables(contents: &Value) -> Result<(String, String), serde_json::Error> {
    let flat_token_list = convert_to_flat_list(&contents, vec![], vec![]);
    let output = flat_token_list
        .iter()
        .map(|(prefix_list, value)| {
            let css_varname = prefix_list
                .iter()
                .map(|key_name| key_name.to_case(Case::Kebab))
                .collect::<Vec<String>>()
                .join("-");
            return format!("${}: {};", css_varname, value.to_string());
        })
        .collect::<Vec<String>>()
        .join("\n");

    return Ok(("scss".to_string(), output));
}

fn convert_to_flat_list(
    value: &Value,
    value_list: FlatTokenList,
    prefix_list: Vec<String>,
) -> FlatTokenList {
    let mut new_value_list = value_list.to_vec();

    let mut new_token_values: FlatTokenList = match value {
        Value::Null => Vec::new(),

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

        Value::String(ref str) => vec![(prefix_list, str.to_string())],

        Value::Number(ref num) => vec![(prefix_list, num.to_string())],

        _ => vec![(prefix_list, value.to_string())],
    };

    new_value_list.append(&mut new_token_values);

    return new_value_list;
}
