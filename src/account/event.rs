use cqrs_es::DomainEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
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

impl DomainEvent for Event {
    fn event_type(&self) -> String {
        let event_type: &str = match self {
            Event::AccountOpened { .. } => "AccountOpened",
            Event::AccountClosed { .. } => "AccountClosed",
            Event::DepositedMoney { .. } => "CustomerDepositedMoney",
            Event::WithdrewMoney { .. } => "CustomerWithdrewCash",
        };
        event_type.to_string()
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}
