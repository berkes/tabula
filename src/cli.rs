use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "tabula")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Namespace,
}

impl Cli {
    /// Returns the ledger file path of this [`Cli`]. A temp workaround to keep the deep digging in
    /// subcommands local to this module
    pub fn ledger_file_path(&self) -> String {
        match &self.command {
            Namespace::Invoices(namespace_args) => match &namespace_args.command {
                InvoiceActions::Create(create_args) => create_args.ledger_file_path.clone(),
                _ => String::new(),
            },
        }
    }
}

#[derive(Debug, Subcommand)]
enum Namespace {
    Invoices(InvoicesArgs),
}

#[derive(Debug, Args)]
struct InvoicesArgs {
    #[command(subcommand)]
    command: InvoiceActions,
}

#[derive(Debug, Subcommand)]
enum InvoiceActions {
    Create(CreateInvoiceArgs),
    Update,
    List,
    Delete,
}

#[derive(Debug, Args)]
struct CreateInvoiceArgs {
    ledger_file_path: String,
}

pub fn parse() -> Cli {
    Cli::parse()
}
