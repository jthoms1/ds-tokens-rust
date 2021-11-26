use serde_json::Value;
use std::str::FromStr;
use strum_macros::{EnumString, EnumVariantNames};

type FlatTokenList = Vec<(Vec<String>, String)>;

#[derive(Debug, EnumString, EnumVariantNames)]
#[strum(serialize_all = "lowercase")]
pub enum Transform {
    CSS,
    SCSS,
    JSON,
    TS,
}

pub fn transform_tokens(file_type: &String, contents: &serde_json::Value) -> String {
    let flat_token_list = convert_to_flat_list(&contents, vec![], vec![]);
    let results = match Transform::from_str(file_type).unwrap() {
        Transform::JSON => json(&contents).unwrap(),
        Transform::TS => typescript(&contents).unwrap(),
        Transform::CSS => css_variables(&flat_token_list).unwrap(),
        Transform::SCSS => scss_variables(&flat_token_list).unwrap(),
    };
    return results;
}

fn json(contents: &Value) -> Result<String, serde_json::Error> {
    let output = format!("{:#}", contents);
    return Ok(output);
}

fn typescript(contents: &Value) -> Result<String, serde_json::Error> {
    let output = format!(
        "
export const themeData = {:#} as const;\n
export type ThemeType = typeof themeData;
  ",
        contents
    );
    return Ok(output);
}

fn scss_variables(flat_token_list: &FlatTokenList) -> Result<String, serde_json::Error> {
    let output = flat_token_list
        .iter()
        .map(|(prefix_list, value)| {
            let css_varname = prefix_list
                .iter()
                .map(|key_name| {
                    inflector::cases::kebabcase::to_kebab_case(
                        &inflector::string::singularize::to_singular(&key_name),
                    )
                })
                .collect::<Vec<String>>()
                .join("-");
            return format!("${}: {};", css_varname, value.to_string());
        })
        .collect::<Vec<String>>()
        .join("\n");

    return Ok(output);
}

fn css_variables(flat_token_list: &FlatTokenList) -> Result<String, serde_json::Error> {
    let results = flat_token_list
        .iter()
        .map(|(prefix_list, value)| {
            let css_varname = prefix_list
                .iter()
                .map(|key_name| {
                    inflector::cases::kebabcase::to_kebab_case(
                        &inflector::string::singularize::to_singular(&key_name),
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

    return Ok(output);
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
