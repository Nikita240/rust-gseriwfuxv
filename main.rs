use std::error::Error;
use std::io;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,

    #[serde(rename = "client")]
    client_id: u16,

    #[serde(rename = "tx")]
    id: u32,

    amount: Decimal,
}

#[derive(Debug, Serialize)]
struct Account {
    #[serde(rename = "client")]
    id: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool
}

impl Account {
    fn deposit(&mut self, amount: Decimal) {
        self.total += amount;
        self.available += amount;
    }

    fn withdraw(&mut self, amount: Decimal) {
        if(self.available - amount >= Decimal::ZERO) {
            self.total -= amount;
            self.available -= amount;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {

    let mut accounts = HashMap::new();

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path("transactions.csv")?;

    for result in rdr.deserialize() {

        let transaction: Transaction = result?;
        println!("{:?}", transaction);

        let account = accounts.entry(transaction.client_id).or_insert(Account {
            id: transaction.client_id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false
        });

        match transaction.transaction_type {
            TransactionType::Deposit => account.deposit(transaction.amount),
            TransactionType::Withdrawal => account.withdraw(transaction.amount),
            _ => ()
        }
    }

    let mut wtr = csv::Writer::from_writer(io::stdout());

    for account in accounts.values() {
        println!("{:?}", account);
        // wtr.serialize(account)?;
    }

    wtr.flush()?;

    Ok(())
}
