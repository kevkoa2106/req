use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Copy)]
enum TokenType {
    Word,
    IntLit,
    Link,
    JsonBody,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
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
                    token_type: TokenType::JsonBody,
                    value: Some(next_line.to_string()),
                });
            }
            continue;
        }

        for word in line.split_whitespace() {
            if word.parse::<f64>().is_ok() {
                tokens.push(Token {
                    token_type: TokenType::IntLit,
                    value: Some(word.to_string()),
                });
            } else if word.contains("http://") {
                tokens.push(Token {
                    token_type: TokenType::Link,
                    value: Some(word.to_string()),
                });
            } else if word.chars().any(|c| c.is_ascii_alphabetic()) {
                tokens.push(Token {
                    token_type: TokenType::Word,
                    value: Some(word.to_string()),
                });
            }
        }
    }

    tokens
}

pub async fn process(
    client: reqwest::Client,
    tokens: Vec<Token>,
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
        "POST" => Ok(client.post(url).send().await?.text().await?),
        _ => Err(format!("unsupported method: {method}").into()),
    }
}
