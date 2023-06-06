use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "tabula")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Namespace,
}

impl Cli {}

#[derive(Debug, Subcommand)]
pub enum Namespace {
    Invoices(InvoicesArgs),
}

#[derive(Debug, Args)]
pub struct InvoicesArgs {
    #[command(subcommand)]
    pub command: InvoiceActions,
}

#[derive(Debug, Subcommand)]
pub enum InvoiceActions {
    Create,
    Update,
    List,
    Delete,
    Build,
}

pub fn parse() -> Cli {
    Cli::parse()
}
