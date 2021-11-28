use serde_json::Value;
use strum_macros::{Display, EnumString, EnumVariantNames};

type FlatTokenListItem = (Vec<String>, String);

#[derive(Debug, EnumString, EnumVariantNames, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Transform {
    Css,
    Scss,
    Json,
    Ts,
}
pub trait Process {
    fn process(&self, contents: &Value) -> Result<String, serde_json::Error>;
}

impl Process for Transform {
    fn process(&self, contents: &Value) -> Result<String, serde_json::Error> {
        match self {
            Transform::Json => process_json(contents),
            Transform::Ts => process_ts(contents),
            Transform::Scss => {
                let flat_token_list = convert_to_flat_list(contents, vec![], vec![]);
                process_scss(&flat_token_list)
            },
            Transform::Css => {
                let flat_token_list = convert_to_flat_list(contents, vec![], vec![]);
                process_css(&flat_token_list)
            }
        }
    }
}

fn process_json(contents: &Value) -> Result<String, serde_json::Error> {
    let output = format!("{:#}", contents);
    Ok(output)
}

fn process_ts(contents: &Value) -> Result<String, serde_json::Error> {
    let output = format!(
        "
export const themeData = {:#} as const;\n
export type ThemeType = typeof themeData;
  ",
        contents
    );
    Ok(output)
}

fn process_scss(flat_token_list: &[(Vec<String>, String)]) -> Result<String, serde_json::Error> {
    let output = flat_token_list
        .iter()
        .map(|(prefix_list, value)| {
            let css_varname = prefix_list
                .iter()
                .map(|key_name| {
                    inflector::cases::kebabcase::to_kebab_case(
                        &inflector::string::singularize::to_singular(key_name),
                    )
                })
                .collect::<Vec<String>>()
                .join("-");
            return format!("${}: {};", css_varname, value.to_string());
        })
        .collect::<Vec<String>>()
        .join("\n");

    Ok(output)
}

fn process_css(flat_token_list: &[(Vec<String>, String)]) -> Result<String, serde_json::Error> {
    let results = flat_token_list
        .iter()
        .map(|(prefix_list, value)| {
            let css_varname = prefix_list
                .iter()
                .map(|key_name| {
                    inflector::cases::kebabcase::to_kebab_case(
                        &inflector::string::singularize::to_singular(key_name),
                    )
                })
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

    Ok(output)
}

fn convert_to_flat_list(
    value: &Value,
    value_list: Vec<FlatTokenListItem>,
    prefix_list: Vec<String>,
) -> Vec<FlatTokenListItem> {
    let mut new_value_list = value_list.to_vec();

    let mut new_token_values: Vec<FlatTokenListItem> = match value {
        Value::Null => Vec::new(),
        Value::String(ref str) => vec![(prefix_list, str.to_string())],
        Value::Number(ref num) => vec![(prefix_list, num.to_string())],
        Value::Array(ref array_values) => array_values
            .iter()
            .enumerate()
            .flat_map(|(index, x)| {
                let mut newvec = prefix_list.to_vec();
                newvec.push((index + 1).to_string());
                convert_to_flat_list(x, value_list.to_vec(), newvec)
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

    new_value_list
}
