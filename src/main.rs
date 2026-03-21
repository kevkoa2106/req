use req::{
    arg_parser,
    parser::{load_env_vars, process, substitute, tokenize},
    tui,
};

#[tokio::main]
async fn main() {
    let args = arg_parser::get_args();

    if !args.filename.contains(".rest") && !args.filename.contains(".http") {
        eprintln!("file not in correct extension");
        return;
    }

    let mut tokens = tokenize(&args.filename);

    if args.tui {
        if let Err(e) = tui::run(tokens).await {
            eprintln!("TUI error: {e}");
        }
        return;
    }

    let vars = load_env_vars(&args.filename, &args.env, args.private);
    for token in &mut tokens {
        token.value = substitute(&token.value, &vars);
    }

    let client = reqwest::Client::new();
    let response = match process(client, &tokens).await {
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
