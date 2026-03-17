use crate::parser::Token;
use reqwest::header::{CONTENT_TYPE, HeaderMap, USER_AGENT};

pub async fn send_post_req(
    client: reqwest::Client,
    tokens: &Vec<Token>,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut header_map = HeaderMap::new();
    let mut content = String::new();

    get_content_and_headers(tokens, &mut header_map, &mut content)?;

    Ok(client
        .post(url)
        .headers(header_map)
        .body(content)
        .send()
        .await?
        .text()
        .await?)
}

pub async fn send_put_req(
    client: reqwest::Client,
    tokens: &Vec<Token>,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut header_map = HeaderMap::new();
    let mut content = String::new();

    get_content_and_headers(tokens, &mut header_map, &mut content)?;

    Ok(client
        .put(url)
        .headers(header_map)
        .body(content)
        .send()
        .await?
        .text()
        .await?)
}

pub async fn send_delete_req(
    client: reqwest::Client,
    tokens: &Vec<Token>,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut header_map = HeaderMap::new();
    let mut content = String::new();

    if tokens.len() > 2 {
        get_content_and_headers(tokens, &mut header_map, &mut content)?;
    }

    Ok(client
        .delete(url)
        .headers(header_map)
        .body(content)
        .send()
        .await?
        .text()
        .await?)
}

pub async fn send_patch_req(
    client: reqwest::Client,
    tokens: &Vec<Token>,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut header_map = HeaderMap::new();
    let mut content = String::new();

    if tokens.len() > 2 {
        get_content_and_headers(tokens, &mut header_map, &mut content)?;
    }

    Ok(client
        .patch(url)
        .headers(header_map)
        .body(content)
        .send()
        .await?
        .text()
        .await?)
}

fn get_content_and_headers(
    tokens: &Vec<Token>,
    header_map: &mut HeaderMap,
    content: &mut String,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(header) = tokens.get(2) {
        let header_val = tokens
            .get(3)
            .and_then(|t| t.value.as_deref())
            .ok_or("missing header value")?;

        match header.value.as_deref().unwrap_or_default() {
            "Content-Type:" => {
                header_map.insert(CONTENT_TYPE, header_val.parse()?);
            }
            "User-Agent:" => {
                header_map.insert(USER_AGENT, header_val.parse()?);
            }
            error => return Err(format!("unsupported header: {error}").into()),
        }
    } else {
        return Err("missing header".into());
    }

    *content = if tokens.len() == 5 {
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

    Ok(())
}
