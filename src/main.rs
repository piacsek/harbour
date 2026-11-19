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
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}
