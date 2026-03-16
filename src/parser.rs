use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::requests::{send_delete_req, send_patch_req, send_post_req, send_put_req};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Token {
    pub value: Option<String>,
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
        "PUT" => Ok(send_put_req(client, &tokens, url).await?),
        "DELETE" => Ok(send_delete_req(client, &tokens, url).await?),
        "PATCH" => Ok(send_patch_req(client, &tokens, url).await?),
        _ => Err(format!("unsupported method: {method}").into()),
    }
}
