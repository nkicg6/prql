use prql_compiler::{compile, sql};
use regex::Regex;
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Base {
    showcase_section: ShowcaseSection,
    #[serde(flatten)]
    _extras: HashMap<String, Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct ShowcaseSection {
    examples: Vec<Examples>,
    #[serde(flatten)]
    _extras: HashMap<String, Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct Examples {
    id: String,
    label: String,
    prql: String,
    sql: String,
}

fn main() {
    // remove `---` so serde_yaml can parse (the hugo front matter looks like a multi-document yaml
    // which isn't supported).
    let hugo_index = include_str!("../../content/_index.md").replace("---", "");
    let as_yml: Base = serde_yaml::from_str(&hugo_index).expect("failed to parse");
    let examples = as_yml.showcase_section.examples;
    let opt = sql::Options {
        format: false,
        signature_comment: false,
        dialect: None,
    };
    let mut failures = 0;
    let mut success = 0;
    let re = Regex::new(r"\s?\s+").expect("Regex failed to compile");
    for example in examples {
        let prql = example.prql;
        let sql_truth = re.replace_all(&example.sql, " ");
        let compiled = match compile(&prql, Some(opt.clone())) {
            Ok(s) => s,
            Err(e) => {
                println!("Compile Error: '{}'", e);
                "".to_string()
            }
        };
        let compiled_normalized = re.replace_all(&compiled, " ");
        if sql_truth.trim() != compiled_normalized.trim() {
            println!("Failed on ID: '{}'", example.id);
            println!("Expected: {}", sql_truth.trim());
            println!("Got:      {}\n---\n", compiled_normalized.trim());
            failures += 1
        } else {
            println!("OK on ID: '{}'\n---\n", example.id);
            success += 1;
        }
    }
    println!("Passed: {}, Failed: {}", success, failures);
}
