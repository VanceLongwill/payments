use anyhow::Result;
use rust_decimal::prelude::*;
use thiserror::Error;

use crate::transactions::{Transaction, TransactionKind};

#[derive(Error, Debug, PartialEq)]
pub enum AccountError {
    #[error("insufficient funds")]
    InsufficientFunds,
    #[error("client ID mismatch")]
    InvalidClient,
    #[error("account must be opened with a deposit transaction")]
    InvalidInitialTransaction,
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
    /// new creates an account from a deposit transaction
    pub fn new(transaction: Transaction) -> Result<Account, AccountError> {
        match transaction.kind {
            TransactionKind::Deposit { amount } => Ok(Account {
                client: transaction.client,
                available: amount,
                held: Decimal::from(0),
                locked: LockedStatus::Unlocked,
            }),
            _ => Err(AccountError::InvalidInitialTransaction),
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
        if self.client != client {
            return Err(AccountError::InvalidClient);
        }
        if self.is_locked() {
            return Err(AccountError::InsufficientFunds);
        }
        match kind {
            TransactionKind::Deposit { .. } => {
                self.available = self.available + amount;
                Ok(())
            }
            TransactionKind::Withdrawal { .. } => {
                let available = self.available - amount;
                if available < Decimal::from(0) {
                    return Err(AccountError::InsufficientFunds.into());
                }
                self.available = available;
                Ok(())
            }
            // @TODO: should dispute, resolve & chargeback transactions error when:
            //      a) the resulting available balance would be negative
            //      b) the resulting held balance would be negative ?
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
    use crate::transactions::TransactionCommand;

    #[test]
    fn test_new_account() -> Result<()> {
        let transaction = Transaction {
            tx: 1,
            kind: TransactionKind::Withdrawal {
                amount: Decimal::from(8),
            },
            client: 1,
            amount: Decimal::from(8),
        };

        let acc = Account::new(transaction);
        assert!(acc.is_err());
        assert_eq!(acc.unwrap_err(), AccountError::InvalidInitialTransaction,);
        Ok(())
    }

    #[test]
    fn test_apply_deposit() -> Result<()> {
        let transaction = Transaction::try_from(TransactionCommand {
            tx: 1,
            kind: TransactionKind::Deposit {
                amount: Decimal::from(8),
            },
            client: 1,
        })?;
        let mut acc = Account::new(transaction)?;
        let amount = Decimal::from(7);
        acc.apply(Transaction {
            client: acc.client,
            tx: 1,
            kind: TransactionKind::Deposit { amount },
            amount,
        })?;
        assert_eq!(acc.available(), Decimal::from(15));
        Ok(())
    }

    #[test]
    fn test_apply_withdrawal() -> Result<()> {
        let transaction = Transaction::try_from(TransactionCommand {
            tx: 1,
            kind: TransactionKind::Deposit {
                amount: Decimal::from(0),
            },
            client: 1,
        })?;
        let mut acc = Account::new(transaction)?;
        acc.available = Decimal::from(8);
        let amount = Decimal::from(7);
        acc.apply(Transaction {
            tx: 1,
            client: acc.client,
            kind: TransactionKind::Withdrawal { amount },
            amount,
        })?;
        assert_eq!(acc.available(), Decimal::from(1));
        Ok(())
    }

    #[test]
    fn test_apply_withdrawal_insufficient_funds() -> Result<()> {
        let transaction = Transaction::try_from(TransactionCommand {
            tx: 1,
            kind: TransactionKind::Deposit {
                amount: Decimal::from(0),
            },
            client: 1,
        })?;
        let mut acc = Account::new(transaction)?;
        acc.available = Decimal::from(8);
        let amount = Decimal::from(10);
        let res = acc.apply(Transaction {
            tx: 1,
            client: acc.client,
            kind: TransactionKind::Withdrawal { amount },
            amount,
        });
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), AccountError::InsufficientFunds);
        assert_eq!(acc.available(), Decimal::from(8));
        Ok(())
    }

    #[test]
    fn test_apply_dispute() -> Result<()> {
        let transaction = Transaction::try_from(TransactionCommand {
            tx: 1,
            kind: TransactionKind::Deposit {
                amount: Decimal::from(0),
            },
            client: 1,
        })?;
        let mut acc = Account::new(transaction)?;
        acc.available = Decimal::from(8);
        let amount = Decimal::from(7);
        acc.apply(Transaction {
            tx: 1,
            client: acc.client,
            kind: TransactionKind::Dispute,
            amount,
        })?;
        assert_eq!(acc.available(), Decimal::from(1));
        assert_eq!(acc.held(), Decimal::from(amount));
        Ok(())
    }

    #[test]
    fn test_apply_resolve() -> Result<()> {
        let transaction = Transaction::try_from(TransactionCommand {
            tx: 1,
            kind: TransactionKind::Deposit {
                amount: Decimal::from(0),
            },
            client: 1,
        })?;
        let mut acc = Account::new(transaction)?;
        acc.held = Decimal::from(7);
        acc.available = Decimal::from(1);
        let amount = Decimal::from(7);
        acc.apply(Transaction {
            tx: 1,
            client: acc.client,
            kind: TransactionKind::Resolve,
            amount,
        })?;
        assert_eq!(acc.available(), Decimal::from(8));
        assert_eq!(acc.held(), Decimal::from(0));
        Ok(())
    }

    #[test]
    fn test_apply_chargeback() -> Result<()> {
        let transaction = Transaction::try_from(TransactionCommand {
            tx: 1,
            kind: TransactionKind::Deposit {
                amount: Decimal::from(0),
            },
            client: 1,
        })?;
        let mut acc = Account::new(transaction)?;
        acc.held = Decimal::from(7);
        acc.available = Decimal::from(1);
        let amount = Decimal::from(2);
        acc.apply(Transaction {
            tx: 1,
            client: acc.client,
            kind: TransactionKind::ChargeBack,
            amount,
        })?;
        assert_eq!(acc.available(), Decimal::from(1));
        assert_eq!(acc.held(), Decimal::from(5));
        assert!(acc.is_locked());
        Ok(())
    }

    #[test]
    fn test_apply_locked() -> Result<()> {
        let transaction = Transaction::try_from(TransactionCommand {
            tx: 1,
            kind: TransactionKind::Deposit {
                amount: Decimal::from(100),
            },
            client: 1,
        })?;
        let mut acc = Account::new(transaction)?;
        acc.locked = LockedStatus::Locked;
        let amount = Decimal::from(10);
        let res = acc.apply(Transaction {
            tx: 1,
            client: acc.client,
            kind: TransactionKind::Withdrawal { amount },
            amount,
        });
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), AccountError::InsufficientFunds);
        assert_eq!(acc.available(), Decimal::from(100));
        Ok(())
    }

    #[test]
    fn test_apply_with_mismatched_client_id() -> Result<()> {
        let transaction = Transaction::try_from(TransactionCommand {
            tx: 1,
            kind: TransactionKind::Deposit {
                amount: Decimal::from(100),
            },
            client: 1,
        })?;
        let mut acc = Account::new(transaction)?;
        let amount = Decimal::from(10);
        let res = acc.apply(Transaction {
            tx: 1,
            client: acc.client + 1,
            kind: TransactionKind::Withdrawal { amount },
            amount,
        });
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), AccountError::InvalidClient);
        assert_eq!(acc.available(), Decimal::from(100));
        Ok(())
    }
}
