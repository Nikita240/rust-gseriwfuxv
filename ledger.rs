use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

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
pub struct Transaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,

    #[serde(rename = "client")]
    client_id: u16,

    #[serde(rename = "tx")]
    id: u32,

    amount: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct Account {
    #[serde(rename = "client")]
    id: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool
}

enum WithdrawError {
    AccountLockedError,
    InsufficientBalanceError
}

impl Account {
    fn deposit(&mut self, amount: Decimal) {
        self.total += amount;
        self.available += amount;
    }

    fn withdraw(&mut self, amount: Decimal) -> Result<(), WithdrawError> { // return result
        if self.locked {
            return Err(WithdrawError::AccountLockedError)
        }
        else if self.available - amount < Decimal::ZERO {
            return Err(WithdrawError::InsufficientBalanceError)
        }

        self.total -= amount;
        self.available -= amount;

        Ok(())
    }

    fn hold(&mut self, amount: Decimal) {
        self.held += amount;
        self.available -= amount;
    }

    fn release(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
    }

    fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;
        self.total -= amount;
        self.locked = true;
    }
}

#[derive(Default)]
pub struct Ledger {
    pub accounts: HashMap<u16, Account>,
    pub transactions: HashMap<u32, Transaction>,
    pub disputed_transactions: HashSet<u32>,
}

impl Ledger {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn transact(&mut self, transaction: Transaction) {
        let account = self.accounts.entry(transaction.client_id).or_insert(Account {
            id: transaction.client_id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false
        });

        match transaction.transaction_type {
            TransactionType::Deposit => {
                account.deposit(transaction.amount.unwrap());
                self.transactions.insert(transaction.id, transaction);
            },
            TransactionType::Withdrawal => {
                match account.withdraw(transaction.amount.unwrap()) {
                    Ok(_) => {
                        self.transactions.insert(transaction.id, transaction);
                    },
                    Err(error) => {
                        match error {
                            WithdrawError::AccountLockedError => eprintln!("Cannot withdraw {} from account {} because the account is locked", transaction.amount.unwrap(), transaction.client_id),
                            WithdrawError::InsufficientBalanceError => eprintln!("Cannot withdraw {} from account {} because the account only has {}", transaction.amount.unwrap(), transaction.client_id, account.available)
                        };
                    }
                };
            },
            TransactionType::Dispute => {
                match self.transactions.get(&transaction.id) {
                    Some(found_transaction) => {
                        account.hold(found_transaction.amount.unwrap());
                        self.disputed_transactions.insert(found_transaction.id);
                    },
                    None => eprintln!("Cannot dispute transaction {} because it does not exist", transaction.id)
                };
            },
            TransactionType::Resolve => {
                match self.disputed_transactions.get(&transaction.id) {
                    Some(_) => match self.transactions.get(&transaction.id) {
                        Some(found_transaction) => {
                            account.release(found_transaction.amount.unwrap());
                            self.disputed_transactions.remove(&found_transaction.id);
                        },
                        None => eprintln!("Cannot resolve transaction {} because it does not exist", transaction.id)
                    },
                    None => eprintln!("Cannot resolve transaction {} because it is not disputed", transaction.id)
                };
            },
            TransactionType::Chargeback => {
                match self.disputed_transactions.get(&transaction.id) {
                    Some(_) => match self.transactions.get(&transaction.id) {
                        Some(found_transaction) => {
                            account.chargeback(found_transaction.amount.unwrap());
                            self.disputed_transactions.remove(&found_transaction.id);
                        },
                        None => eprintln!("Cannot chargeback transaction {} because it does not exist", transaction.id)
                    },
                    None => eprintln!("Cannot chargeback transaction {} because it is not disputed", transaction.id)
                };
            }
        };
    }
}
