mod cli;
mod invoice;
mod output;

fn main() {
    let output = match cli::parse().command {
        cli::Namespace::Invoices(invoices_args) => match invoices_args.command {
            cli::InvoiceActions::Create => invoice::handle_create(),
            cli::InvoiceActions::Build => invoice::handle_build(),
            cli::InvoiceActions::List => invoice::handle_list(),
            cli::InvoiceActions::Convert(args) => invoice::handle_convert(args),
        },
    };
    println!("{}", output);
}
