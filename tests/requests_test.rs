use std::collections::HashMap;

use reqwest::header::{ACCEPT, HOST, USER_AGENT};
use wiremock::matchers::{body_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use req::parser::{Token, TokenType};
use req::requests::{send_delete_req, send_patch_req, send_post_req, send_put_req};

fn make_headers_map() -> HashMap<&'static str, reqwest::header::HeaderName> {
    HashMap::from([
        ("Host", HOST),
        ("User-Agent", USER_AGENT),
        ("Accept", ACCEPT),
    ])
}

#[tokio::test]
async fn post_with_body_and_header() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/data"))
        .and(header("User-Agent", "test-agent"))
        .and(body_string(r#"{"name": "test"}"#))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id": 1}"#))
        .mount(&server)
        .await;

    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "POST".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: format!("{}/api/data", server.uri()),
        },
        Token {
            token_type: TokenType::Header,
            value: "User-Agent".to_string(),
        },
        Token {
            token_type: TokenType::HeaderValue,
            value: "test-agent".to_string(),
        },
        Token {
            token_type: TokenType::Body,
            value: r#"{"name": "test"}"#.to_string(),
        },
    ];

    let client = reqwest::Client::new();
    let headers = make_headers_map();
    let result = send_post_req(
        client,
        &tokens,
        &format!("{}/api/data", server.uri()),
        &headers,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), r#"{"id": 1}"#);
}

#[tokio::test]
async fn post_with_host_header() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/items"))
        .and(header("Host", "example.com"))
        .respond_with(ResponseTemplate::new(201).set_body_string("created"))
        .mount(&server)
        .await;

    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "POST".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: format!("{}/items", server.uri()),
        },
        Token {
            token_type: TokenType::Header,
            value: "Host".to_string(),
        },
        Token {
            token_type: TokenType::HeaderValue,
            value: "example.com".to_string(),
        },
        Token {
            token_type: TokenType::Body,
            value: "payload".to_string(),
        },
    ];

    let client = reqwest::Client::new();
    let headers = make_headers_map();
    let result = send_post_req(
        client,
        &tokens,
        &format!("{}/items", server.uri()),
        &headers,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "created");
}

#[tokio::test]
async fn post_empty_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/empty"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "POST".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: format!("{}/empty", server.uri()),
        },
    ];

    let client = reqwest::Client::new();
    let headers = make_headers_map();
    let result = send_post_req(
        client,
        &tokens,
        &format!("{}/empty", server.uri()),
        &headers,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "ok");
}

#[tokio::test]
async fn post_server_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/fail"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal server error"))
        .mount(&server)
        .await;

    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "POST".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: format!("{}/fail", server.uri()),
        },
        Token {
            token_type: TokenType::Body,
            value: "data".to_string(),
        },
    ];

    let client = reqwest::Client::new();
    let headers = make_headers_map();
    let result = send_post_req(client, &tokens, &format!("{}/fail", server.uri()), &headers).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("500"), "error should contain status code");
    assert!(
        err_msg.contains("internal server error"),
        "error should contain response body"
    );
}

#[tokio::test]
async fn post_server_error_verbose_has_debug_info() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/fail"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&server)
        .await;

    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "POST".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: format!("{}/fail", server.uri()),
        },
        Token {
            token_type: TokenType::Body,
            value: "data".to_string(),
        },
    ];

    let client = reqwest::Client::new();
    let headers = make_headers_map();
    let result = send_post_req(client, &tokens, &format!("{}/fail", server.uri()), &headers).await;

    assert!(result.is_err());
    let err = result.unwrap_err();

    // Display format (non-verbose): short message
    let display = err.to_string();
    assert!(display.contains("404"));
    assert!(display.contains("not found"));

    // Debug format (verbose): includes more detail
    let debug = format!("{err:?}");
    assert!(debug.contains("404"));
    assert!(debug.contains("not found"));
}

#[tokio::test]
async fn post_unknown_header_is_ignored() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/data"))
        .respond_with(ResponseTemplate::new(200).set_body_string("done"))
        .mount(&server)
        .await;

    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "POST".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: format!("{}/data", server.uri()),
        },
        Token {
            token_type: TokenType::Header,
            value: "X-Custom-Unknown".to_string(),
        },
        Token {
            token_type: TokenType::HeaderValue,
            value: "some-value".to_string(),
        },
        Token {
            token_type: TokenType::Body,
            value: "body".to_string(),
        },
    ];

    let client = reqwest::Client::new();
    let headers = make_headers_map();
    let result = send_post_req(client, &tokens, &format!("{}/data", server.uri()), &headers).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "done");
}

#[tokio::test]
async fn post_connection_refused() {
    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "POST".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: "http://127.0.0.1:1".to_string(),
        },
        Token {
            token_type: TokenType::Body,
            value: "data".to_string(),
        },
    ];

    let client = reqwest::Client::new();
    let headers = make_headers_map();
    let result = send_post_req(client, &tokens, "http://127.0.0.1:1", &headers).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn put_with_body() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/1"))
        .and(body_string(r#"{"name": "updated"}"#))
        .respond_with(ResponseTemplate::new(200).set_body_string("updated"))
        .mount(&server)
        .await;

    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "PUT".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: format!("{}/api/1", server.uri()),
        },
        Token {
            token_type: TokenType::Body,
            value: r#"{"name": "updated"}"#.to_string(),
        },
    ];

    let client = reqwest::Client::new();
    let headers = make_headers_map();
    let result = send_put_req(
        client,
        &tokens,
        &format!("{}/api/1", server.uri()),
        &headers,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "updated");
}

#[tokio::test]
async fn delete_simple() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/1"))
        .respond_with(ResponseTemplate::new(200).set_body_string("deleted"))
        .mount(&server)
        .await;

    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "DELETE".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: format!("{}/api/1", server.uri()),
        },
    ];

    let client = reqwest::Client::new();
    let headers = make_headers_map();
    let result = send_delete_req(
        client,
        &tokens,
        &format!("{}/api/1", server.uri()),
        &headers,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "deleted");
}

#[tokio::test]
async fn patch_with_body() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/1"))
        .and(body_string(r#"{"name": "patched"}"#))
        .respond_with(ResponseTemplate::new(200).set_body_string("patched"))
        .mount(&server)
        .await;

    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "PATCH".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: format!("{}/api/1", server.uri()),
        },
        Token {
            token_type: TokenType::Body,
            value: r#"{"name": "patched"}"#.to_string(),
        },
    ];

    let client = reqwest::Client::new();
    let headers = make_headers_map();
    let result = send_patch_req(
        client,
        &tokens,
        &format!("{}/api/1", server.uri()),
        &headers,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "patched");
}
