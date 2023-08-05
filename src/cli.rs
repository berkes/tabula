use clap::{Args, Parser, Subcommand, ValueEnum};

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
    List,
    /// Converts to --format of an invoice in a ledger
    Convert(ConvertArgs),

    /// Builds a template invoice JSON file
    Build,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum OutputFormat {
    Json,
    Txt,
    Pdf,
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    /// The format to convert to
    #[arg(long)]
    pub format: OutputFormat,
    /// The invoice number. If multiple invoices are found, the first one is used.
    #[arg(long)]
    pub invoice_number: String,
}

pub fn parse() -> Cli {
    Cli::parse()
}
