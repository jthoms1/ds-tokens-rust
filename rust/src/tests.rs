use serde_json::Value;

use ds_tokens_rust::{Process, Transform};
use strum::VariantNames;

#[test]
fn transform_type() {
    let default_values: Vec<String> =
            Transform::VARIANTS
            .to_vec()
            .into_iter()
            .map(|str| str.to_owned())
            .collect();

    assert_eq!(default_values, vec!("css", "scss", "json", "ts"));
}

#[test]
fn test_css_process() {
    let build_string = "
    fontWeights:
        regular: 400
        medium: 500
        semibold: 600
        bold: 700
    ";
    let contents: Value = serde_yaml::from_str(build_string).unwrap();
    let actual_results = Transform::CSS.process(&contents).unwrap();
    let expected_results = "
:root { 
  --font-weight-bold: 700;
  --font-weight-medium: 500;
  --font-weight-regular: 400;
  --font-weight-semibold: 600;
}
";
    assert_eq!(actual_results,expected_results);
}

#[test]
fn test_scss_process() {
    let build_string = "
    fontWeights:
        regular: 400
        medium: 500
        semibold: 600
        bold: 700
    ";
    let contents: Value = serde_yaml::from_str(build_string).unwrap();
    let actual_results = Transform::SCSS.process(&contents).unwrap();
    let expected_results = 
"$font-weight-bold: 700;
$font-weight-medium: 500;
$font-weight-regular: 400;
$font-weight-semibold: 600;";
    assert_eq!(actual_results,expected_results);
}