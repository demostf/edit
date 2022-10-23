use clap::Parser;
use edit::{edit_inner, EditOptions};
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the source demo
    path: String,
    #[arg(long)]
    unlock_pov: bool,

}

fn main() {
    let args = Args::parse();
    let file = fs::read(&args.path).unwrap();
    let output = edit_inner(&file, EditOptions::default());
    fs::write("out.dem", output).unwrap();
}
