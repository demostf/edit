use clap::Parser;
use edit::{edit_inner, EditOptions, TickRange};
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the source demo
    path: String,
    #[arg(long)]
    unlock_pov: bool,
    #[arg(long)]
    from: Option<u32>,
    #[arg(long)]
    to: Option<u32>,
}

impl Args {
    fn get_options(&self) -> EditOptions {
        EditOptions {
            unlock_pov: self.unlock_pov,
            cut: if let (Some(from), Some(to)) = (self.from, self.to) {
                Some(TickRange { from: from.into(), to: to.into() })
            } else {
                None
            },
            ..EditOptions::default()
        }
    }
}

fn main() {
    let args: Args = Args::parse();
    let options = args.get_options();
    let file = fs::read(&args.path).unwrap();
    let output = edit_inner(&file, options);
    fs::write("out.dem", output).unwrap();
}
