use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "aliasman")]
#[command(version)]
#[command(about = "Manage shell aliases safely")]
struct Cli {}

fn main() {
    Cli::parse();
}
