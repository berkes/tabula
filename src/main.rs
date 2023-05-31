use account::aggregate::Account;
use async_trait::async_trait;
use beancount_core::{
    directives::{Directive, Transaction},
    posting::Posting,
    Ledger,
};
use beancount_parser::parse;
use beancount_render::render;
use cqrs_es::{EventEnvelope, Query};
use std::borrow::Cow;
use std::{fs::File, io::Read, panic};

mod account;

fn main() {
    let ledger_content = read_file("/home/ber/tmp/test.beancount");
    let ledger = parse(&ledger_content).unwrap();
    let transactions: Vec<Transaction> = ledger
        .directives
        .into_iter()
        .filter_map(|item| {
            if let Directive::Transaction(tx) = item {
                Some(tx)
            } else {
                None
            }
        })
        .collect();

    report(&tax_ledger(transactions.clone()), "Tax report");
    report(&invoices_ledger(transactions), "Invoices");
}

fn report(ledger: &Ledger, header: &str) {
    let mut report = Vec::new();
    render(&mut report, ledger).unwrap();

    println!("=={:=<78}", format!(" {} ", header));
    println!("{}", String::from_utf8(report).unwrap());
}

fn invoices_ledger(transactions: Vec<Transaction>) -> Ledger {
    let directives = find_transactions_by_meta_key(transactions, "invoice_number")
        .into_iter()
        .map(Directive::Transaction)
        .collect();
    Ledger { directives }
}

fn tax_ledger(transactions: Vec<Transaction>) -> Ledger {
    let directives = find_transactions_by_account(transactions, "Omzetbelasting")
        .into_iter()
        .map(Directive::Transaction)
        .collect();
    Ledger { directives }
}

fn read_file(path: &str) -> String {
    let mut file = match File::open(path) {
        Err(why) => panic!("Couldn't open {}: {}", path, why),
        Ok(file) => file,
    };

    let mut contents = String::new();
    if let Err(why) = file.read_to_string(&mut contents) {
        panic!("Couldn't read {}: {}", path, why)
    }
    contents
}

fn find_transactions_by_meta_key<'a>(
    transactions: Vec<Transaction<'a>>,
    meta_key: &'a str,
) -> Vec<Transaction<'a>> {
    transactions
        .into_iter()
        .filter_map(|transaction| -> Option<Transaction> {
            transaction
                .meta
                .contains_key(meta_key)
                .then_some(transaction)
        })
        .collect()
}

fn find_transactions_by_account<'a>(
    transactions: Vec<Transaction<'a>>,
    posting_account_contains: &'a str,
) -> Vec<Transaction<'a>> {
    transactions
        .into_iter()
        .filter_map(|transaction| -> Option<Transaction> {
            let owned_postings: Vec<Posting> = transaction.postings.clone();
            if owned_postings.into_iter().any(|posting| {
                posting
                    .account
                    .parts
                    .contains(&Cow::Borrowed(posting_account_contains))
            }) {
                Some(transaction)
            } else {
                None
            }
        })
        .collect()
}

pub struct SimpleLoggingQuery {}
#[async_trait]
impl Query<Account> for SimpleLoggingQuery {
    async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<Account>]) {
        for event in events {
            println!(
                "{}-{}\n#{:#?}",
                aggregate_id, &event.sequence, &event.payload
            )
        }
    }
}

#[cfg(test)]
mod main_test {
    use cqrs_es::{mem_store::MemStore, CqrsFramework};

    use crate::{
        account::{
            aggregate::Account, command::Command as AccountCommand,
            services::Services as AccountServices,
        },
        SimpleLoggingQuery,
    };

    #[tokio::test]
    #[should_panic]
    async fn test_event_store() {
        let event_store = MemStore::<Account>::default();
        let query = SimpleLoggingQuery {};
        let cqrs = CqrsFramework::new(event_store, vec![Box::new(query)], AccountServices);

        let aggregate_id = "aggregate-instance-A";

        // open account
        cqrs.execute(
            aggregate_id,
            AccountCommand::OpenAccount {
                account_id: "LEET0 1337".to_string(),
            },
        )
        .await
        .unwrap();

        // deposit $1000
        cqrs.execute(
            aggregate_id,
            AccountCommand::DepositMoney {
                amount: 1000_f64,
                currency: "EUR".to_string(),
            },
        )
        .await
        .unwrap();

        // withdraw $236.15
        cqrs.execute(
            aggregate_id,
            AccountCommand::WithdrawMoney {
                amount: 236.15,
                currency: "EUR".to_string(),
            },
        )
        .await
        .unwrap();

        // overdraw.
        cqrs.execute(
            aggregate_id,
            AccountCommand::WithdrawMoney {
                amount: 770.00,
                currency: "EUR".to_string(),
            },
        )
        .await
        .unwrap();
    }
}
