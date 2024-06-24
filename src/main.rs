use arboard::Clipboard;
use colored::Colorize;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use url::Url;

const RULES_FILE_PATH: &str = "./src/rules.json";
const TYPE_BLACKLIST: u8 = 0;
const TYPE_WHITELIST: u8 = 1;

#[derive(Deserialize, Debug)]
struct Rule {
    domain: Value,
    #[serde(rename = "type")]
    type_field: u8,
    path: String,
    params: Vec<String>,
}

fn main() {
    let mut clipboard = Clipboard::new().expect("Failed to initialize clipboard");

    let file = fs::read_to_string(&RULES_FILE_PATH).expect("Failed to read file");

    let rules_json: Vec<Rule> = serde_json::from_str(&file).expect("JSON is malformed");

    let input_url = match clipboard.get_text() {
        Ok(text) => text,
        Err(e) => {
            eprintln!("{} {}", "Failed to read clipboard".red(), e);
            return;
        }
    };

    println!("{} {}", "Input:".blue(), input_url.trim().bright_black());

    let input_url = match Url::parse(&input_url.trim()) {
        Ok(url) => url,
        Err(e) => {
            eprintln!("{} {}", "Failed to parse URL".red(), e);
            return;
        }
    };

    let mut input_url_host = match &input_url.host() {
        Some(h) => h.to_string(),
        None => String::from("No host found"),
    };

    if input_url_host.starts_with("www.") {
        input_url_host = input_url_host[4..].to_string();
    }

    let input_url_path = input_url.path();

    let mut stripped_url = Url::parse(&input_url.to_string()).unwrap();

    stripped_url = Url::parse(&format!(
        "https://{}{}",
        stripped_url.host_str().expect("URL does not have a host"),
        stripped_url.path()
    ))
    .expect("Failed to parse new URL");

    let mut applied_params = HashSet::new();

    for rule in &rules_json {
        let rule_domain = rule.domain.to_string().replace("\"", "");

        if input_url_host == rule_domain {
            if rule.path != "*" && rule.path != input_url_path {
                continue;
            }

            println!("{} {}", "Detected URL:".green(), rule_domain);

            for (key, val) in input_url.query_pairs() {
                if rule.type_field == TYPE_BLACKLIST {
                    if !rule.params.contains(&key.to_string())
                        && !applied_params.contains(&key.to_string())
                    {
                        stripped_url
                            .query_pairs_mut()
                            .append_pair(&key.to_string(), &val.to_string());
                        applied_params.insert(key.to_string());
                    }
                } else if rule.type_field == TYPE_WHITELIST {
                    if rule.params.contains(&key.to_string())
                        && !applied_params.contains(&key.to_string())
                    {
                        stripped_url
                            .query_pairs_mut()
                            .append_pair(&key.to_string(), &val.to_string());
                        applied_params.insert(key.to_string());
                    }
                }
            }

            break;
        } else if rule.domain == true {
            for (key, val) in input_url.query_pairs() {
                if rule.type_field == TYPE_BLACKLIST {
                    if !rule.params.contains(&key.to_string())
                        && !applied_params.contains(&key.to_string())
                    {
                        stripped_url
                            .query_pairs_mut()
                            .append_pair(&key.to_string(), &val.to_string());
                        applied_params.insert(key.to_string());
                    }
                } else if rule.type_field == TYPE_WHITELIST {
                    if rule.params.contains(&key.to_string())
                        && !applied_params.contains(&key.to_string())
                    {
                        stripped_url
                            .query_pairs_mut()
                            .append_pair(&key.to_string(), &val.to_string());
                        applied_params.insert(key.to_string());
                    }
                }
            }
        }
    }

    clipboard
        .set_text(stripped_url.to_string())
        .expect("Failed to copy text to clipboard");

    println!("{}:", "✔️ Successfully stripped URL".green());
    println!("{}", stripped_url.to_string().blue().underline());
    println!("{}", "📋 Copied stripped URL to clipboard".green());
}
