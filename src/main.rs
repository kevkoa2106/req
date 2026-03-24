use req::{
    arg_parser,
    parser::{load_env_vars, process_all, substitute, tokenize},
    tui,
};

#[tokio::main]
async fn main() {
    let args = arg_parser::get_args();

    if !args.filename.contains(".rest") && !args.filename.contains(".http") {
        eprintln!("file not in correct extension");
        return;
    }

    let mut requests = tokenize(&args.filename);

    let vars = load_env_vars(&args.filename, &args.env, args.private);
    for request in &mut requests {
        for token in request.iter_mut() {
            token.value = substitute(&token.value, &vars);
        }
    }

    if args.tui {
        if let Err(e) = tui::run(requests).await {
            eprintln!("TUI error: {e}");
        }
        return;
    }

    let client = reqwest::Client::new();
    let results = process_all(client, &requests).await;

    for (i, result) in results.into_iter().enumerate() {
        if i > 0 {
            println!("\n###\n");
        }
        let response = match result {
            Ok(c) => c,
            Err(e) => {
                if args.verbose {
                    format!("{e:?}")
                } else {
                    e.to_string()
                }
            }
        };

        match formatjson::format_json(response.as_str()) {
            Ok(formatted) => println!("{formatted}"),
            Err(_) => println!("{response}"),
        }
    }
}
