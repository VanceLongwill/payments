use anyhow::{anyhow, Result};
use rust_decimal::prelude::*;
use serde::Serialize;

use crate::transactions::{Transaction, TransactionKind};

#[derive(PartialEq, Debug, Clone)]
enum LockedStatus {
    Locked,
    Unlocked,
}

impl Serialize for LockedStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(match self {
            LockedStatus::Locked => true,
            LockedStatus::Unlocked => false,
        })
    }
}

#[derive(Debug)]
pub struct Account {
    available: Decimal,
    held: Decimal,
    locked: LockedStatus,
}

impl Account {
    pub fn new() -> Account {
        Account {
            available: Decimal::from(0),
            held: Decimal::from(0),
            locked: LockedStatus::Unlocked,
        }
    }
    pub fn available(&self) -> Decimal {
        self.available
    }
    pub fn held(&self) -> Decimal {
        self.held
    }
    pub fn total(&self) -> Decimal {
        self.available + self.held
    }
    pub fn is_locked(&self) -> bool {
        self.locked == LockedStatus::Locked
    }
    pub fn apply(&mut self, Transaction { kind, amount, .. }: Transaction) -> Result<()> {
        if self.is_locked() {
            return Err(anyhow!("account is locked"));
        }
        match kind {
            TransactionKind::Deposit { .. } => {
                self.available = self.available + amount;
                Ok(())
            }
            TransactionKind::Withdrawal { .. } => {
                if self.available - amount < Decimal::from(0) {
                    return Err(anyhow!("insufficient funds"));
                }
                self.available = self.available - amount;
                Ok(())
            }
            TransactionKind::Dispute => {
                self.available = self.available - amount;
                self.held = self.held + amount;
                Ok(())
            }
            TransactionKind::Resolve => {
                self.held = self.held - amount;
                self.available = self.available + amount;
                Ok(())
            }
            TransactionKind::ChargeBack => {
                self.held = self.held - amount;
                self.available = self.available - amount;
                self.locked = LockedStatus::Locked;
                Ok(())
            }
        }
    }
}
