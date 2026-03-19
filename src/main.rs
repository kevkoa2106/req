use req::{
    arg_parser,
    parser::{process, tokenize},
};

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let args = arg_parser::get_args();

    let tokens = tokenize(&args.filename);

    let response = if args.filename.contains(".rest") || args.filename.contains(".http") {
        match process(client, &tokens).await {
            Ok(c) => c,
            Err(e) => {
                if args.verbose {
                    format!("{e:?}")
                } else {
                    e.to_string()
                }
            }
        }
    } else {
        String::from("file not in correct extension")
    };

    match formatjson::format_json(response.as_str()) {
        Ok(formatted) => println!("{formatted}"),
        Err(_) => println!("{response}"),
    }
}
