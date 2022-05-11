use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

pub mod transaction;
use transaction::{Transaction, TransactionType};

pub mod account;
use account::{Account, WithdrawError};

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
                account.deposit(transaction.amount.unwrap_or(Decimal::ZERO));
                self.transactions.insert(transaction.id, transaction);
            },
            TransactionType::Withdrawal => {
                match account.withdraw(transaction.amount.unwrap_or(Decimal::ZERO)) {
                    Ok(_) => {
                        self.transactions.insert(transaction.id, transaction);
                    },
                    Err(error) => {
                        match error {
                            WithdrawError::AccountLockedError => eprintln!("Cannot withdraw {} from account {} because the account is locked", transaction.amount.unwrap_or(Decimal::ZERO), transaction.client_id),
                            WithdrawError::InsufficientBalanceError => eprintln!("Cannot withdraw {} from account {} because the account only has {}", transaction.amount.unwrap_or(Decimal::ZERO), transaction.client_id, account.available)
                        };
                    }
                };
            },
            TransactionType::Dispute => {
                match self.transactions.get(&transaction.id) {
                    Some(found_transaction) => {
                        account.hold(found_transaction.amount.unwrap_or(Decimal::ZERO));
                        self.disputed_transactions.insert(found_transaction.id);
                    },
                    None => eprintln!("Cannot dispute transaction {} because it does not exist", transaction.id)
                };
            },
            TransactionType::Resolve => {
                match self.disputed_transactions.get(&transaction.id) {
                    Some(_) => match self.transactions.get(&transaction.id) {
                        Some(found_transaction) => {
                            account.release(found_transaction.amount.unwrap_or(Decimal::ZERO));
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
                            account.chargeback(found_transaction.amount.unwrap_or(Decimal::ZERO));
                            self.disputed_transactions.remove(&found_transaction.id);

                            // In a typical production system, it probably makes sense to classify Chargebacks
                            // as a new transaction. That way you avoid doing a "double chargeback" without
                            // having to actually remove the transaction from your history.
                        },
                        None => eprintln!("Cannot chargeback transaction {} because it does not exist", transaction.id)
                    },
                    None => eprintln!("Cannot chargeback transaction {} because it is not disputed", transaction.id)
                };
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit_withdraw() {
        let mut ledger = Ledger::new();
        ledger.transact(Transaction {
            transaction_type: TransactionType::Deposit,
            client_id: 0,
            id: 0,
            amount: Some(Decimal::new(10, 0))
        });
        ledger.transact(Transaction {
            transaction_type: TransactionType::Withdrawal,
            client_id: 0,
            id: 1,
            amount: Some(Decimal::new(5, 0))
        });
        assert_eq!(ledger.accounts.get(&0).unwrap().total, Decimal::new(5, 0));
    }

    #[test]
    fn withdraw_insufficient_funds() {
        let mut ledger = Ledger::new();
        ledger.transact(Transaction {
            transaction_type: TransactionType::Deposit,
            client_id: 0,
            id: 0,
            amount: Some(Decimal::new(10, 0))
        });
        ledger.transact(Transaction {
            transaction_type: TransactionType::Withdrawal,
            client_id: 0,
            id: 1,
            amount: Some(Decimal::new(15, 0))
        });
        assert_eq!(ledger.accounts.get(&0).unwrap().total, Decimal::new(10, 0));
    }

    #[test]
    fn resolve_dispute() {
        // Note, the logic specified in the requirements doesn't actually make sense.
        // It states that we should should hold funds from the client no matter if the
        // disputed transaction is a deposit or a withdrawal. Surely if a client disputes
        // a withdrawal, we should put the withdrawn amount into the "hold" state instead
        // of deducting it from the current available amount? Currently, if a client
        // disputes withdrawal, it's basically like asking to double charge him.
        //
        // Quote: "This means that the clients available funds should decrease by the amount
        //         disputed, their held funds should increase by the amount disputed, while
        //         their total funds should remain the same."
        //
        // It's very explicit about "decrease" and "increase" while making no mention of the
        // type of transaction being disputed.

        let mut ledger = Ledger::new();
        ledger.transact(Transaction {
            transaction_type: TransactionType::Deposit,
            client_id: 0,
            id: 0,
            amount: Some(Decimal::new(10, 0))
        });
        ledger.transact(Transaction {
            transaction_type: TransactionType::Dispute,
            client_id: 0,
            id: 0,
            amount: None
        });
        assert_eq!(ledger.accounts.get(&0).unwrap().total, Decimal::new(10, 0));
        assert_eq!(ledger.accounts.get(&0).unwrap().available, Decimal::new(0, 0));
        assert_eq!(ledger.accounts.get(&0).unwrap().held, Decimal::new(10, 0));
        ledger.transact(Transaction {
            transaction_type: TransactionType::Resolve,
            client_id: 0,
            id: 0,
            amount: None
        });
        assert_eq!(ledger.accounts.get(&0).unwrap().total, Decimal::new(10, 0));
        assert_eq!(ledger.accounts.get(&0).unwrap().available, Decimal::new(10, 0));
        assert_eq!(ledger.accounts.get(&0).unwrap().held, Decimal::new(0, 0));
    }

    #[test]
    fn chargeback_dispute() {
        let mut ledger = Ledger::new();
        ledger.transact(Transaction {
            transaction_type: TransactionType::Deposit,
            client_id: 0,
            id: 0,
            amount: Some(Decimal::new(10, 0))
        });
        ledger.transact(Transaction {
            transaction_type: TransactionType::Dispute,
            client_id: 0,
            id: 0,
            amount: None
        });
        assert_eq!(ledger.accounts.get(&0).unwrap().total, Decimal::new(10, 0));
        assert_eq!(ledger.accounts.get(&0).unwrap().available, Decimal::new(0, 0));
        assert_eq!(ledger.accounts.get(&0).unwrap().held, Decimal::new(10, 0));
        ledger.transact(Transaction {
            transaction_type: TransactionType::Chargeback,
            client_id: 0,
            id: 0,
            amount: None
        });
        assert_eq!(ledger.accounts.get(&0).unwrap().total, Decimal::new(0, 0));
        assert_eq!(ledger.accounts.get(&0).unwrap().available, Decimal::new(0, 0));
        assert_eq!(ledger.accounts.get(&0).unwrap().held, Decimal::new(0, 0));
    }
}