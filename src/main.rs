use account::aggregate::Account;
use async_trait::async_trait;
use cqrs_es::{EventEnvelope, Query};
use ledger_parser::{LedgerItem, Transaction};
use std::{fs::File, io::Read, panic};

mod account;

fn main() {
    let ledger_content = read_file("/home/ber/tmp/test.ledger");
    let ledger = ledger_parser::parse(&ledger_content).unwrap();
    let summary: Vec<Transaction> = ledger
        .items
        .into_iter()
        .filter_map(|item| {
            if let LedgerItem::Transaction(tx) = item {
                Some(tx)
            } else {
                None
            }
        })
        .collect();

    let vat_summary = filter_transactions(summary, "btw");

    for line in vat_summary {
        println!("{}", line);
    }
}

fn read_file(path: &str) -> String {
    let mut file = match File::open(path) {
        Err(why) => panic!("Couldn't open {}: {}", path, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("Couldn't read {}: {}", path, why),
        Ok(_) => {}
    }
    return s;
}

fn filter_transactions(
    transactions: Vec<Transaction>,
    posting_account_contains: &str,
) -> Vec<Transaction> {
    transactions
        .into_iter()
        .filter_map(|transaction| -> Option<Transaction> {
            if transaction
                .postings
                .iter()
                .any(|posting| posting.account.contains(posting_account_contains))
            {
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
