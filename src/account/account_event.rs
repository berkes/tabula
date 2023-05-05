use cqrs_es::DomainEvent;
use serde::{Serialize, Deserialize};

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
        balance: f64,
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

