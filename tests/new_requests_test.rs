use std::collections::HashMap;

use reqwest::header::{ACCEPT, HOST, USER_AGENT};
use wiremock::matchers::{body_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use req::new_requests::send_post_req;
use req::parser_new::{Token, TokenType};

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
    let result = send_post_req(client, &tokens, &format!("{}/api/data", server.uri()), &headers).await;

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
    let result = send_post_req(client, &tokens, &format!("{}/items", server.uri()), &headers).await;

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

    // No header or body tokens — just method and URL
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
    let result = send_post_req(client, &tokens, &format!("{}/empty", server.uri()), &headers).await;

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

    // The function returns the response body even on 500
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "internal server error");
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
            value: "X-Custom-Unknown".to_string(), // not in our headers map
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

    // Unknown header is silently skipped, request still succeeds
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
            value: "http://127.0.0.1:1".to_string(), // nothing listening
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
