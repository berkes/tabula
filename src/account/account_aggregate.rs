use async_trait::async_trait;
use cqrs_es::Aggregate;
use serde::{Deserialize, Serialize};

use super::{
    account_command::AccountCommand,
    account_errors::AccountError,
    account_event::AccountEvent,
    account_services::AccountServices,
};

#[derive(Serialize, Default, Deserialize)]
pub struct Account {
    account_id: String,
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
            AccountCommand::OpenAccount { account_id } => {
                Ok(vec![AccountEvent::AccountOpened { account_id }])
            }
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
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            AccountEvent::AccountOpened { account_id } => {
                self.opened = true;
                self.account_id = account_id;
            }
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
