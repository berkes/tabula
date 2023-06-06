mod cli;
mod invoice;

fn main() {
    let output = match cli::parse().command {
        cli::Namespace::Invoices(invoices_args) => match invoices_args.command {
            cli::InvoiceActions::Create => invoice::handle_create(),
            cli::InvoiceActions::Build => invoice::handle_build(),
            _ => String::new(),
        },
    };
    println!("{}", output);
}
