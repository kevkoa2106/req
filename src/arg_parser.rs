use clap::{Arg, ArgAction, arg, command};

pub struct Args {
    pub filename: String,
    pub verbose: bool,
    pub tui: bool,
}

pub fn get_args() -> Args {
    let matches = command!()
        .version("v0.1.0") // requires `cargo` feature
        .about("Simple example with positional args")
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
        .get_matches();

    let filename = matches.get_one::<String>("INPUT").expect("required").into();
    let verbose = matches.get_flag("verbose");
    let tui = matches.get_flag("tui");

    Args {
        filename,
        verbose,
        tui,
    }
}
