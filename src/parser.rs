use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
    sync::LazyLock,
};

use regex::Regex;
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION, CONTENT_TYPE, HOST,
    HeaderName, IF_MODIFIED_SINCE, IF_NONE_MATCH, REFERER, UPGRADE_INSECURE_REQUESTS, USER_AGENT,
};

use crate::requests::{send_delete_req, send_patch_req, send_post_req, send_put_req};

static HEADERS: LazyLock<HashMap<&str, HeaderName>> = LazyLock::new(|| {
    let headers = HashMap::from([
        ("Host", HOST),
        ("User-Agent", USER_AGENT),
        ("Accept", ACCEPT),
        ("Accept-Language", ACCEPT_LANGUAGE),
        ("Accept-Encoding", ACCEPT_ENCODING),
        ("Referer", REFERER),
        ("Connection", CONNECTION),
        ("Upgrade-Insecure-Requests", UPGRADE_INSECURE_REQUESTS),
        ("If-Modified-Since", IF_MODIFIED_SINCE),
        ("If-None-Match", IF_NONE_MATCH),
        ("Cache-Control", CACHE_CONTROL),
        ("Content-Type", CONTENT_TYPE),
    ]);
    headers
});

#[derive(Debug, Clone)]
pub enum TokenType {
    Method,
    URL,
    Header,
    HeaderValue,
    Body,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
}

pub fn tokenize(filename: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let reader = BufReader::new(File::open(filename).expect("Cannot open file"));

    let mut lines = reader.lines().peekable();

    while let Some(Ok(line)) = lines.next() {
        if line.trim().is_empty() {
            let mut body = String::new();
            for remaining in lines.by_ref() {
                if let Ok(l) = remaining {
                    if !body.is_empty() {
                        body.push(' ');
                    }
                    body.push_str(l.trim());
                }
            }
            if !body.is_empty() {
                tokens.push(Token {
                    token_type: TokenType::Body,
                    value: body,
                });
            }
            break;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            if HEADERS.contains_key(key) {
                tokens.push(Token {
                    token_type: TokenType::Header,
                    value: key.to_string(),
                });
                tokens.push(Token {
                    token_type: TokenType::HeaderValue,
                    value: value.trim().to_string(),
                });
                continue;
            }
        }

        for word in line.split_whitespace() {
            if word.contains("GET")
                || word.contains("POST")
                || word.contains("PUT")
                || word.contains("DELETE")
            {
                tokens.push(Token {
                    token_type: TokenType::Method,
                    value: word.to_string(),
                });
            } else if word.starts_with("http://") || word.starts_with("https://") {
                tokens.push(Token {
                    token_type: TokenType::URL,
                    value: word.to_string(),
                });
            } else {
            }
        }
    }

    tokens
}

pub async fn process(
    client: reqwest::Client,
    tokens: &Vec<Token>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut method = String::new();
    let mut url = String::new();

    for token in tokens {
        match token.token_type {
            TokenType::Method => method = token.value.clone(),
            TokenType::URL => url = token.value.clone(),
            _ => (),
        }
    }

    match method.as_str() {
        "GET" => Ok(client.get(url.clone()).send().await?.text().await?),
        "POST" => Ok(send_post_req(client, &tokens, url.as_str(), &HEADERS).await?),
        "PUT" => Ok(send_put_req(client, &tokens, url.as_str(), &HEADERS).await?),
        "DELETE" => Ok(send_delete_req(client, &tokens, url.as_str(), &HEADERS).await?),
        "PATCH" => Ok(send_patch_req(client, &tokens, url.as_str(), &HEADERS).await?),
        _ => Err(format!("unsupported method: {method}").into()),
    }
}

pub fn load_env_vars(
    rest_file: &str,
    environment: &str,
    use_private: bool,
) -> HashMap<String, String> {
    let dir = Path::new(rest_file).parent().unwrap_or(Path::new("."));

    // Load base env
    let mut vars = load_env_file(&dir.join("http-client.env.json"), environment);

    // Private overrides base
    if use_private {
        let private = load_env_file(&dir.join("http-client.private.env.json"), environment);
        vars.extend(private); // keys in private overwrite keys in base
    }

    vars
}

fn load_env_file(path: &Path, environment: &str) -> HashMap<String, String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };

    let envs: HashMap<String, HashMap<String, serde_json::Value>> =
        serde_json::from_str(&content).unwrap_or_default();

    envs.get(environment)
        .map(|vars| {
            vars.iter()
                .map(|(k, v)| {
                    let val = match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    (k.clone(), val)
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn substitute(input: &str, vars: &HashMap<String, String>) -> String {
    let re = Regex::new(r"\{\{(\s*[\w\-]+\s*)\}\}").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        let key = caps[1].trim();
        vars.get(key)
            .cloned()
            .unwrap_or_else(|| caps[0].to_string()) // leave unresolved
    })
    .to_string()
}
