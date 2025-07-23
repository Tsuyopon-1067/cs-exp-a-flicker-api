use clap::Parser;
mod preprocess;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the CSV file to process
    #[arg(short, long)]
    prepare: Option<String>,

    /// A flag to indicate hashing
    #[arg(short = 'H', long)]
    hash: bool,
}

fn main() {
    let args = Args::parse();

    if let Some(path) = args.prepare {
        println!("'--prepare' or '-p' option is provided with path: {}", path);
        if let Err(e) = preprocess::preprocess(&path) {
            eprintln!("Error during preprocessing: {}", e);
        }
    } else {
        println!("'--prepare' or '-p' option is NOT provided.");
    }

    if args.hash {
        println!("'--hash' or '-H' option is provided.");
    } else {
        println!("'--hash' or '-H' option is NOT provided.");
    }
}
