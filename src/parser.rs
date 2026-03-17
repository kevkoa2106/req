use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION, HOST, IF_MODIFIED_SINCE,
    IF_NONE_MATCH, REFERER, UPGRADE_INSECURE_REQUESTS, USER_AGENT,
};

use crate::requests::{send_delete_req, send_patch_req, send_post_req, send_put_req};

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
    ]);

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
                    body.push_str(&l);
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
            if headers.contains_key(key) {
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
            } else if word.starts_with("http://") {
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
    ]);

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
        "POST" => Ok(send_post_req(client, &tokens, url.as_str(), &headers).await?),
        "PUT" => Ok(send_put_req(client, &tokens, url.as_str(), &headers).await?),
        "DELETE" => Ok(send_delete_req(client, &tokens, url.as_str(), &headers).await?),
        "PATCH" => Ok(send_patch_req(client, &tokens, url.as_str(), &headers).await?),
        _ => Err(format!("unsupported method: {method}").into()),
    }
}
