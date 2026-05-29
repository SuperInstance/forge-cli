mod commands;
mod output;

use std::env;
use std::process;

fn usage() -> ! {
    eprintln!("Usage: forge <command> [args...]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  detect <file>                    — detect input format");
    eprintln!("  decompose <file> [--format FMT]  — decompose into tiles (prints JSON)");
    eprintln!("  compose <tiles.json>             — reassemble tiles back");
    eprintln!("  stats <file>                     — show tile statistics");
    eprintln!("  pipeline <config.json>           — run a pipeline");
    eprintln!("  list                             — list all forge-* crates in ecosystem");
    eprintln!("  info <crate-name>                — show crate details");
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
    }

    let result = match args[1].as_str() {
        "detect" => commands::detect(&args[2..]),
        "decompose" => commands::decompose(&args[2..]),
        "compose" => commands::compose(&args[2..]),
        "stats" => commands::stats(&args[2..]),
        "pipeline" => commands::pipeline(&args[2..]),
        "list" => commands::list(&args[2..]),
        "info" => commands::info(&args[2..]),
        "--help" | "-h" => {
            usage();
        }
        other => {
            eprintln!("Unknown command: {}", other);
            usage();
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
