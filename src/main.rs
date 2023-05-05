use account::account_aggregate::Account;
use async_trait::async_trait;
use cqrs_es::{Query, EventEnvelope};

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
mod aggregate_tests {
    use crate::account::{account_command::AccountCommand::{DepositMoney, self}, account_services::AccountServices, account_event::AccountEvent, account_errors::AccountError};
    use super::*;
    use cqrs_es::{mem_store::MemStore, test::TestFramework, CqrsFramework};

    type AccountTestFramework = TestFramework<Account>;

    #[test]
    fn test_deposit_money() {
        let expected = AccountEvent::DepositedMoney {
            amount: 200.0,
            currency: "EUR".to_string(),
            balance: 200.0,
        };

        AccountTestFramework::with(AccountServices)
            .given_no_previous_events()
            .when(DepositMoney {
                amount: 200.0,
                currency: "EUR".to_string(),
            })
            .then_expect_events(vec![expected]);
    }

    #[test]
    fn test_deposit_money_with_balance() {
        let previous = AccountEvent::DepositedMoney {
            amount: 200.0,
            balance: 200.0,
            currency: "EUR".to_string(),
        };
        let expected = AccountEvent::DepositedMoney {
            amount: 200.0,
            balance: 400.0,
            currency: "EUR".to_string(),
        };

        AccountTestFramework::with(AccountServices)
            .given(vec![previous])
            .when(DepositMoney {
                amount: 200.0,
                currency: "EUR".to_string(),
            })
            .then_expect_events(vec![expected]);
    }

    #[test]
    fn test_withdraw_money() {
        let previous = AccountEvent::DepositedMoney {
            amount: 200.0,
            balance: 200.0,
            currency: "EUR".to_string(),
        };
        let expected = AccountEvent::WithdrewMoney {
            amount: 100.0,
            balance: 100.0,
            currency: "EUR".to_string(),
        };

        AccountTestFramework::with(AccountServices)
            .given(vec![previous])
            .when(AccountCommand::WithdrawMoney {
                amount: 100.0,
                currency: "EUR".to_string(),
            })
            .then_expect_events(vec![expected]);
    }

    #[test]
    fn test_withdraw_money_funds_not_available() {
        AccountTestFramework::with(AccountServices)
            .given_no_previous_events()
            .when(AccountCommand::WithdrawMoney {
                amount: 200.0,
                currency: "EUR".to_string(),
            })
            .then_expect_error(AccountError::from("funds not available"));
    }

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
