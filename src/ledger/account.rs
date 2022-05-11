use serde::{Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Serialize)]
pub struct Account {
    #[serde(rename = "client")]
    pub id: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool
}

pub enum WithdrawError {
    AccountLockedError,
    InsufficientBalanceError
}

impl Account {
    pub fn deposit(&mut self, amount: Decimal) {
        self.total += amount;
        self.available += amount;
    }

    pub fn withdraw(&mut self, amount: Decimal) -> Result<(), WithdrawError> { // return result
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

    pub fn hold(&mut self, amount: Decimal) {
        self.held += amount;
        self.available -= amount;
    }

    pub fn release(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
    }

    pub fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;
        self.total -= amount;
        self.locked = true;
    }
}