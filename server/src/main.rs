use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// A flag to indicate preparation
    #[arg(short, long)]
    prepare: bool,

    /// A flag to indicate hashing
    #[arg(short = 'H', long)]
    hash: bool,
}

fn main() {
    let args = Args::parse();

    if args.prepare {
        println!("'--prepare' or '-p' option is provided.");
    } else {
        println!("'--prepare' or '-p' option is NOT provided.");
    }

    if args.hash {
        println!("'--hash' or '-H' option is provided.");
    } else {
        println!("'--hash' or '-H' option is NOT provided.");
    }
}
