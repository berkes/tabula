use beancount_core::{directives::Transaction, Date, Flag, Ledger};
use beancount_render::render;
use clap::{Args, Parser, Subcommand};
use std::{fs::OpenOptions, io::Write};

#[derive(Debug, Parser)]
#[command(name = "tabula")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Namespace,
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

fn main() {
    let args = Cli::parse();

    let path = match args.command {
        Namespace::Invoices(namespace_args) => match namespace_args.command {
            InvoiceActions::Create(create_args) => create_args.ledger_file_path,
            _ => String::new(),
        },
    };

    let today: Date<'static> = chrono::Local::now().date_naive().into();

    let tx = Transaction::builder()
        .flag(Flag::Warning)
        .date(today)
        .narration("Invoice #1".into())
        .build();

    let ledger = Ledger {
        directives: vec![beancount_core::Directive::Transaction(tx)],
    };

    let mut w = Vec::new();
    render(&mut w, &ledger).unwrap();
    let ledger_file_contents = String::from_utf8(w).unwrap();

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();

    if let Err(e) = writeln!(file, "{}", ledger_file_contents) {
        eprintln!("Couldn't write to file: {}", e);
    }

    println!("Invoice #1 created");
}
