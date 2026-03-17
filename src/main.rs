use req::parser_new::{process, tokenize};

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let tokens = tokenize("http.rest");

    let response = match process(client, &tokens).await {
        Ok(c) => c,
        Err(e) => e.to_string(),
    };

    match formatjson::format_json(response.as_str()) {
        Ok(formatted) => println!("{formatted}"),
        Err(_) => println!("{response}"),
    }
}
