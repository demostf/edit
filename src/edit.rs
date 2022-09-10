use clap::Parser;
use edit::{EditOptions, rust_edit};
use std::fs;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the source demo
    path: String,
}

fn main() {
    let args = Args::parse();
    let file = fs::read(&args.path).unwrap();
    let output = rust_edit(&file, EditOptions::default());
    fs::write("out.dem", output).unwrap();
}
