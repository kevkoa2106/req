use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};

use reqwest::header::{CONTENT_TYPE, HeaderMap};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Token {
    value: Option<String>,
}

pub fn tokenize(filename: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let reader = BufReader::new(File::open(filename).expect("Cannot open file"));

    let mut lines = reader.lines().peekable();

    while let Some(Ok(line)) = lines.next() {
        if line.trim().is_empty() {
            if let Some(Ok(next_line)) = lines.next() {
                tokens.push(Token {
                    value: Some(next_line.to_string()),
                });
            }
            continue;
        }

        for word in line.split_whitespace() {
            if word.parse::<f64>().is_ok() {
                tokens.push(Token {
                    value: Some(word.to_string()),
                });
            } else if word.contains("http://") {
                tokens.push(Token {
                    value: Some(word.to_string()),
                });
            } else if word.chars().any(|c| c.is_ascii_alphabetic())
                || word.contains('"')
                || word.contains(',')
            {
                tokens.push(Token {
                    value: Some(word.to_string()),
                });
            } else if word.contains('{') || word.contains('}') {
                tokens.push(Token {
                    value: Some(word.to_string()),
                });
            }
        }
    }

    tokens
}

pub async fn process(
    client: reqwest::Client,
    tokens: &Vec<Token>,
) -> Result<String, Box<dyn std::error::Error>> {
    let method = tokens
        .get(0)
        .ok_or("missing method")?
        .value
        .as_deref()
        .ok_or("empty method")?;
    let url = tokens
        .get(1)
        .ok_or("missing URL")?
        .value
        .as_deref()
        .ok_or("empty URL")?;

    match method {
        "GET" => Ok(client.get(url).send().await?.text().await?),
        "POST" => Ok(send_post_req(client, &tokens, url).await?),
        _ => Err(format!("unsupported method: {method}").into()),
    }
}

async fn send_post_req(
    client: reqwest::Client,
    tokens: &Vec<Token>,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut header_map = HeaderMap::new();

    if let Some(header) = tokens.get(2) {
        let header_val = tokens
            .get(3)
            .and_then(|t| t.value.as_deref())
            .ok_or("missing header value")?;

        match header.value.as_deref().unwrap_or_default() {
            "Content-Type:" => {
                header_map.insert(CONTENT_TYPE, header_val.parse()?);
            }
            error => return Err(format!("unsupported header: {error}").into()),
        }
    } else {
        return Err("missing header".into());
    }

    let content = if tokens.len() == 5 {
        tokens
            .get(4)
            .and_then(|t| t.value.as_deref())
            .ok_or("missing body")?
            .to_string()
    } else if tokens.len() > 5 {
        tokens
            .get(4..)
            .ok_or("missing body")?
            .iter()
            .filter_map(|t| t.value.as_deref())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        return Err("missing body".into());
    };

    Ok(client
        .post(url)
        .headers(header_map)
        .body(content)
        .send()
        .await?
        .text()
        .await?)
}
