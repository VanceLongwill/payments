use anyhow::Result;
use rust_decimal::prelude::*;
use thiserror::Error;

use crate::transactions::{Transaction, TransactionKind};

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("insufficient funds")]
    InsufficientFunds,
}

#[derive(PartialEq, Debug, Clone)]
enum LockedStatus {
    Locked,
    Unlocked,
}

#[derive(Debug)]
pub struct Account {
    client: u16,
    available: Decimal,
    held: Decimal,
    locked: LockedStatus,
}

impl Account {
    pub fn new(client: u16) -> Account {
        Account {
            client,
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
    pub fn apply(
        &mut self,
        Transaction {
            kind,
            amount,
            client,
            ..
        }: Transaction,
    ) -> Result<(), AccountError> {
        if self.client != client {}
        if self.is_locked() {
            return Err(AccountError::InsufficientFunds);
        }
        match kind {
            TransactionKind::Deposit { .. } => {
                self.available = self.available + amount;
                Ok(())
            }
            TransactionKind::Withdrawal { .. } => {
                if self.available - amount < Decimal::from(0) {
                    return Err(AccountError::InsufficientFunds.into());
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
                self.locked = LockedStatus::Locked;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_deposit() -> Result<()> {
        let mut acc = Account::new(1);
        acc.available = Decimal::from(8);
        let amount = Decimal::from(7);
        acc.apply(Transaction {
            tx: 1,
            client: 2,
            kind: TransactionKind::Deposit { amount },
            amount,
        })?;
        assert_eq!(acc.available(), Decimal::from(15));
        Ok(())
    }

    #[test]
    fn test_apply_withdrawal() -> Result<()> {
        let mut acc = Account::new(1);
        acc.available = Decimal::from(8);
        let amount = Decimal::from(7);
        acc.apply(Transaction {
            tx: 1,
            client: 2,
            kind: TransactionKind::Withdrawal { amount },
            amount,
        })?;
        assert_eq!(acc.available(), Decimal::from(1));
        Ok(())
    }

    #[test]
    fn test_apply_withdrawal_insufficient_funds() -> Result<()> {
        let mut acc = Account::new(1);
        acc.available = Decimal::from(8);
        let amount = Decimal::from(10);
        assert!(acc
            .apply(Transaction {
                tx: 1,
                client: 2,
                kind: TransactionKind::Withdrawal { amount },
                amount,
            })
            .is_err());
        assert_eq!(acc.available(), Decimal::from(8));
        Ok(())
    }

    #[test]
    fn test_apply_dispute() -> Result<()> {
        let mut acc = Account::new(1);
        acc.available = Decimal::from(8);
        let amount = Decimal::from(7);
        acc.apply(Transaction {
            tx: 1,
            client: 2,
            kind: TransactionKind::Dispute,
            amount,
        })?;
        assert_eq!(acc.available(), Decimal::from(1));
        assert_eq!(acc.held(), Decimal::from(amount));
        Ok(())
    }

    #[test]
    fn test_apply_resolve() -> Result<()> {
        let mut acc = Account::new(1);
        acc.held = Decimal::from(7);
        acc.available = Decimal::from(1);
        let amount = Decimal::from(7);
        acc.apply(Transaction {
            tx: 1,
            client: 2,
            kind: TransactionKind::Resolve,
            amount,
        })?;
        assert_eq!(acc.available(), Decimal::from(8));
        assert_eq!(acc.held(), Decimal::from(0));
        Ok(())
    }

    #[test]
    fn test_apply_chargeback() -> Result<()> {
        let mut acc = Account::new(1);
        acc.held = Decimal::from(7);
        acc.available = Decimal::from(1);
        let amount = Decimal::from(2);
        acc.apply(Transaction {
            tx: 1,
            client: 2,
            kind: TransactionKind::ChargeBack,
            amount,
        })?;
        assert_eq!(acc.available(), Decimal::from(1));
        assert_eq!(acc.held(), Decimal::from(5));
        Ok(())
    }
}
