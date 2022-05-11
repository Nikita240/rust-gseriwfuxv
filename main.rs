use std::error::Error;
use std::io;
use serde::Deserialize;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    transaction_type: String,
    client: u16,
    tx: u32,
    amount: Decimal,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path("transactions.csv")?;

    for result in rdr.deserialize() {

        let transaction: Transaction = result?;
        println!("{:?}", transaction);
    }
    Ok(())
}
