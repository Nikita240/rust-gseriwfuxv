use std::error::Error;
use std::io;
use std::env;
use std::string::String;

mod ledger;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let filename: &String = &args[1];

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(filename)?;

    let mut ledger = ledger::Ledger::new();

    for result in rdr.deserialize() {

        let transaction: ledger::Transaction = result?;

        ledger.transact(transaction);
    }

    let mut wtr = csv::Writer::from_writer(io::stdout());

    for account in ledger.accounts.values() {
        wtr.serialize(account)?;
    }

    wtr.flush()?;

    Ok(())
}
