use serde::Deserialize;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,

    #[serde(rename = "client")]
    pub client_id: u16,

    #[serde(rename = "tx")]
    pub id: u32,

    pub amount: Option<Decimal>,
}