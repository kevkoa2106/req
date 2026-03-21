use clap::{Arg, ArgAction, arg, command};

pub struct Args {
    pub filename: String,
    pub verbose: bool,
    pub tui: bool,
    pub env: String,
    pub private: bool,
}

pub fn get_args() -> Args {
    let matches = command!()
        .version("v0.1.0") // requires `cargo` feature
        .about("A recreation of IntelliJ's HTTP client in the terminal")
        .arg(arg!(<INPUT> "The input file").required(true))
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Shows full error")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("tui")
                .short('t')
                .long("tui")
                .help("Launch interactive TUI mode")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("env")
                .short('e')
                .long("env")
                .help("Environment name from http-client.env.json (e.g. development, production)")
                .default_value("development"),
        )
        .arg(
            Arg::new("private")
                .long("private")
                .help("Load http-client.private.env.json")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let filename = matches.get_one::<String>("INPUT").expect("required").into();
    let verbose = matches.get_flag("verbose");
    let tui = matches.get_flag("tui");
    let env = matches.get_one::<String>("env").expect("has default").clone();
    let private = matches.get_flag("private");

    Args {
        filename,
        verbose,
        tui,
        env,
        private,
    }
}
