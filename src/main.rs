use beancount_core::{directives::Transaction, Date, Flag, Ledger, Posting, Account, AccountType::Assets, IncompleteAmount};
use beancount_render::render;
use std::{fs::OpenOptions, io::Write, borrow::Cow};

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
        Posting::builder()
            .account(work)
            .units(amt)
            .build()
    ];
    let tx = Transaction::builder()
        .flag(Flag::Warning)
        .date(today)
        .narration("Invoice #1".into())
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

    if let Err(e) = writeln!(file, "{}", ledger_file_contents) {
        eprintln!("Couldn't write to file: {}", e);
    }

    println!("Invoice #1 created");
}
