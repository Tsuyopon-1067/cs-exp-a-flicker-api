use clap::Parser;
mod models;
mod preprocess;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the CSV file to process
    #[arg(short, long, num_args(2))]
    prepare: Option<Vec<String>>,

    /// A flag to indicate hashing
    #[arg(short = 'H', long)]
    hash: bool,
}

fn main() {
    let args = Args::parse();

    if let Some(paths) = args.prepare {
        println!(
            "'--prepare' or '-p' option is provided with paths: {} and {}",
            &paths[0], &paths[1]
        );
        if let Err(e) = preprocess::preprocess(&paths[0], &paths[1]) {
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
