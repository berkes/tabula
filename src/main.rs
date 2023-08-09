mod adapters;
mod commands;
mod domain;
mod services;

use adapters::{cli::CliAdapter, InputAdapter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = CliAdapter::default();
    cli.run()?;
    println!("{}", cli.get_response());
    Ok(())
}
