use std::collections::HashMap;

use reqwest::header::{HeaderMap, HeaderName};

use crate::parser::{Token, TokenType};

pub async fn send_post_req(
    client: reqwest::Client,
    tokens: &Vec<Token>,
    url: &str,
    headers: &HashMap<&str, HeaderName>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut header_map = HeaderMap::new();
    let mut body = String::new();
    let mut current_header: Option<HeaderName> = None;

    get_content_and_headers(
        tokens,
        headers,
        &mut header_map,
        &mut body,
        &mut current_header,
    )?;

    Ok(client
        .post(url)
        .headers(header_map)
        .body(body)
        .send()
        .await?
        .text()
        .await?)
}

pub async fn send_put_req(
    client: reqwest::Client,
    tokens: &Vec<Token>,
    url: &str,
    headers: &HashMap<&str, HeaderName>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut header_map = HeaderMap::new();
    let mut body = String::new();
    let mut current_header: Option<HeaderName> = None;

    get_content_and_headers(
        tokens,
        headers,
        &mut header_map,
        &mut body,
        &mut current_header,
    )?;

    Ok(client
        .put(url)
        .headers(header_map)
        .body(body)
        .send()
        .await?
        .text()
        .await?)
}

pub async fn send_delete_req(
    client: reqwest::Client,
    tokens: &Vec<Token>,
    url: &str,
    headers: &HashMap<&str, HeaderName>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut header_map = HeaderMap::new();
    let mut body = String::new();
    let mut current_header: Option<HeaderName> = None;

    if tokens.len() > 2 {
        get_content_and_headers(
            tokens,
            headers,
            &mut header_map,
            &mut body,
            &mut current_header,
        )?;
    }

    Ok(client
        .delete(url)
        .headers(header_map)
        .body(body)
        .send()
        .await?
        .text()
        .await?)
}

pub async fn send_patch_req(
    client: reqwest::Client,
    tokens: &Vec<Token>,
    url: &str,
    headers: &HashMap<&str, HeaderName>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut header_map = HeaderMap::new();
    let mut body = String::new();
    let mut current_header: Option<HeaderName> = None;

    get_content_and_headers(
        tokens,
        headers,
        &mut header_map,
        &mut body,
        &mut current_header,
    )?;

    Ok(client
        .patch(url)
        .headers(header_map)
        .body(body)
        .send()
        .await?
        .text()
        .await?)
}

fn get_content_and_headers(
    tokens: &Vec<Token>,
    headers: &HashMap<&str, HeaderName>,
    header_map: &mut HeaderMap,
    body: &mut String,
    current_header: &mut Option<HeaderName>,
) -> Result<(), Box<dyn std::error::Error>> {
    for token in tokens {
        match token.token_type {
            TokenType::Header => {
                *current_header = headers.get(token.value.as_str()).cloned();
            }
            TokenType::HeaderValue => {
                if let Some(h) = current_header.take() {
                    header_map.insert(h, token.value.parse()?);
                }
            }
            TokenType::Body => {
                *body = token.value.clone();
            }
            _ => (),
        };
    }

    Ok(())
}
