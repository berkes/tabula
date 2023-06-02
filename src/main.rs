use beancount_core::{
    directives::Transaction,
    metadata::{Meta, MetaValue},
    Account,
    AccountType::Assets,
    Date, Flag, IncompleteAmount, Ledger, Posting,
};
use beancount_render::render;
use std::{borrow::Cow, fs::OpenOptions, io::Write};

mod cli;

fn main() {
    let path = cli::parse().ledger_file_path();

    let today: Date<'static> = chrono::Local::now().date_naive().into();

    let accts_recievable = Account::builder()
        .ty(Assets)
        .parts(vec![Cow::Borrowed("AccountsReceivable")])
        .build();

    let work = Account::builder()
        .ty(beancount_core::AccountType::Income)
        .parts(vec![Cow::Borrowed("Work")])
        .build();

    let amt = IncompleteAmount::builder()
        .num(Some(1337.into()))
        .currency(Some(Cow::Borrowed("USD")))
        .build();

    let postings = vec![
        Posting::builder()
            .account(accts_recievable)
            .units(amt.clone())
            .build(),
        Posting::builder().account(work).units(amt).build(),
    ];

    let invoice_number = "TBD";

    let meta = Meta::from([(
        Cow::Borrowed("invoice_number"),
        MetaValue::Text(invoice_number.into()),
    )]);

    let tx = Transaction::builder()
        .flag(Flag::Warning)
        .meta(meta)
        .date(today)
        .narration(format!("Invoice #{}", invoice_number).into())
        .postings(postings)
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
    writeln!(file, "{}", ledger_file_contents).unwrap();

    println!("Invoice #{} created", invoice_number);
}
