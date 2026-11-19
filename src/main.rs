use std::io::IsTerminal;
use std::process::ExitCode;

use harbour::changelog::CHANGELOG;

const USAGE: &str =
    "harbour: Harbour: where shipping begins\n\nusage: harbour [--help | --version | changelog]";

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args.iter().map(|arg| arg.to_str()).collect::<Vec<_>>()[..] {
        [Some("--version" | "-V" | "version")] => {
            println!("harbour {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [Some("--help" | "-h" | "help")] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        [Some("changelog")] => {
            print!("{CHANGELOG}");
            ExitCode::SUCCESS
        }
        [] if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() => {
            match harbour::tui::run() {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("harbour: {err}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}
