use std::fmt::Display;

use async_trait::async_trait;
use cqrs_es::{Aggregate, DomainEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub enum AccountCommand {
    OpenAccount { account_id: String },
    DepositMoney { amount: f64, currency: String },
    WithdrawMoney { amount: f64, currency: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountEvent {
    AccountOpened {
        account_id: String,
    },
    AccountClosed {
        account_id: String,
    },
    DepositedMoney {
        amount: f64,
        currency: String,
        balance: f64, // TODO: events should best not carry global state
    },
    WithdrewMoney {
        amount: f64,
        currency: String,
        balance: f64,
    },
}

impl DomainEvent for AccountEvent {
    fn event_type(&self) -> String {
        let event_type: &str = match self {
            AccountEvent::AccountOpened { .. } => "AccountOpened",
            AccountEvent::AccountClosed { .. } => "AccountClosed",
            AccountEvent::DepositedMoney { .. } => "CustomerDepositedMoney",
            AccountEvent::WithdrewMoney { .. } => "CustomerWithdrewCash",
        };
        event_type.to_string()
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}

#[derive(Debug, PartialEq)]
pub struct AccountError(String);
impl std::error::Error for AccountError {}
impl Display for AccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<&str> for AccountError {
    fn from(message: &str) -> Self {
        AccountError(message.to_string())
    }
}

pub struct AccountServices;

impl AccountServices {}
pub struct AtmError;

#[derive(Serialize, Default, Deserialize)]
pub struct Account {
    opened: bool,
    // this is a floating point for our example, don't do this IRL
    balance: f64,
}

#[async_trait]
impl Aggregate for Account {
    type Command = AccountCommand;
    type Event = AccountEvent;
    type Error = AccountError;
    type Services = AccountServices;

    fn aggregate_type() -> String {
        "Account".to_string()
    }

    async fn handle(
        &self,
        command: Self::Command,
        _services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            AccountCommand::DepositMoney { amount, currency } => {
                let balance = self.balance + amount;
                Ok(vec![AccountEvent::DepositedMoney {
                    amount,
                    currency,
                    balance,
                }])
            }
            AccountCommand::WithdrawMoney { amount, currency } => {
                let balance = self.balance - amount;
                if balance < 0_f64 {
                    return Err(AccountError::from("funds not available"));
                }
                Ok(vec![AccountEvent::WithdrewMoney {
                    amount,
                    currency,
                    balance,
                }])
            }
            _ => Ok(vec![]),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            AccountEvent::AccountOpened { .. } => self.opened = true,
            AccountEvent::AccountClosed { .. } => self.opened = false,
            AccountEvent::DepositedMoney {
                amount: _,
                currency: _,
                balance,
            } => {
                self.balance = balance;
            }

            AccountEvent::WithdrewMoney {
                amount: _,
                currency: _,
                balance,
            } => {
                self.balance = balance;
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod aggregate_tests {
    use super::*;
    use crate::AccountCommand::DepositMoney;
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
            .then_expect_error(AccountError("funds not available".to_string()));
    }
}
