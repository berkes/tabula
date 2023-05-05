use account::aggregate::Account;
use async_trait::async_trait;
use cqrs_es::{EventEnvelope, Query};

mod account;

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
fn main() {
    println!("Hello, world!");
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
