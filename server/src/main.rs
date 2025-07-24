use clap::Parser;
mod http_server;
mod models;
mod preprocess;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the CSV file to process
    #[arg(short, long, num_args(2))]
    prepare: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Some(paths) = args.prepare {
        if let Err(e) = preprocess::preprocess(&paths[0], &paths[1]) {
            eprintln!("Error during preprocessing: {}", e);
        }
    } else {
        if let Err(e) = http_server::start_server().await {
            eprintln!("Server error: {}", e);
        }
    }
}
