use serde_json::Value;

pub type FlatTokenList = Vec<(Vec<String>, String)>;

pub fn json(contents: &Value) -> Result<String, serde_json::Error> {
    let output = format!("{:#}", contents);
    return Ok(output);
}

pub fn typescript(contents: &Value) -> Result<String, serde_json::Error> {
    let output = format!(
        "
export const themeData = {:#} as const;\n
export type ThemeType = typeof themeData;
  ",
        contents
    );
    return Ok(output);
}

pub fn scss_variables(flat_token_list: &FlatTokenList) -> Result<String, serde_json::Error> {
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

pub fn css_variables(flat_token_list: &FlatTokenList) -> Result<String, serde_json::Error> {
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
