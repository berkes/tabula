use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum Command {
    OpenAccount { account_id: String },
    DepositMoney { amount: f64, currency: String },
    WithdrawMoney { amount: f64, currency: String },
}
