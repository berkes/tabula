use async_trait::async_trait;
use cqrs_es::Aggregate;
use serde::{Deserialize, Serialize};

use super::{command::Command, errors::Error, event::Event, services::Services};

#[derive(Serialize, Default, Deserialize)]
pub struct Account {
    account_id: String,
    opened: bool,
    // this is a floating point for our example, don't do this IRL
    balance: f64,
}

#[async_trait]
impl Aggregate for Account {
    type Command = Command;
    type Event = Event;
    type Error = Error;
    type Services = Services;

    fn aggregate_type() -> String {
        "Account".to_string()
    }

    async fn handle(
        &self,
        command: Self::Command,
        _services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            Command::OpenAccount { account_id } => Ok(vec![Event::AccountOpened { account_id }]),
            Command::DepositMoney { amount, currency } => {
                let balance = self.balance + amount;
                Ok(vec![Event::DepositedMoney {
                    amount,
                    currency,
                    balance,
                }])
            }
            Command::WithdrawMoney { amount, currency } => {
                let balance = self.balance - amount;
                if balance < 0_f64 {
                    return Err(Error::from("funds not available"));
                }
                Ok(vec![Event::WithdrewMoney {
                    amount,
                    currency,
                    balance,
                }])
            }
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            Event::AccountOpened { account_id } => {
                self.opened = true;
                self.account_id = account_id;
            }
            Event::AccountClosed { .. } => self.opened = false,
            Event::DepositedMoney {
                amount: _,
                currency: _,
                balance,
            } => {
                self.balance = balance;
            }

            Event::WithdrewMoney {
                amount: _,
                currency: _,
                balance,
            } => {
                self.balance = balance;
            }
        }
    }
}

#[cfg(test)]
mod aggregate_tests {
    use super::*;
    use crate::account::{
        command::Command as AccountCommand, errors::Error as AccountError,
        event::Event as AccountEvent, services::Services as AccountServices,
    };
    use cqrs_es::test::TestFramework;

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
            .when(AccountCommand::DepositMoney {
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
            .when(AccountCommand::DepositMoney {
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
}
