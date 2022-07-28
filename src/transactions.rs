use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;

use anyhow::{anyhow, Result};
use rust_decimal::prelude::*;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("unable to move transaction from {from:?} to {to:?}")]
    InvalidState {
        from: TransactionKind,
        to: TransactionKind,
    },
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct TransactionCommand {
    #[serde(flatten)]
    pub kind: TransactionKind,
    pub tx: u32,
    pub client: u16,
}

impl TryFrom<TransactionCommand> for Transaction {
    type Error = anyhow::Error;
    fn try_from(
        TransactionCommand { kind, tx, client }: TransactionCommand,
    ) -> Result<Transaction> {
        match kind {
            TransactionKind::Deposit { amount } | TransactionKind::Withdrawal { amount } => {
                Ok(Transaction {
                    tx,
                    amount,
                    kind,
                    client,
                })
            }
            _ => {
                return Err(anyhow!(
                    "transactions must start with a deposit or withdrawal"
                ))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum TransactionKind {
    Deposit { amount: Decimal },
    Withdrawal { amount: Decimal },
    Dispute,
    Resolve,
    ChargeBack,
}

#[derive(Debug, Clone, Copy)]
pub struct Transaction {
    pub tx: u32,
    pub amount: Decimal,
    pub kind: TransactionKind,
    pub client: u16,
}

impl Transaction {
    pub fn apply(&self, kind: TransactionKind) -> Result<Transaction> {
        match (self.kind, kind) {
            (TransactionKind::Deposit { amount }, TransactionKind::Dispute)
            | (TransactionKind::Withdrawal { amount }, TransactionKind::Dispute) => {
                Ok(Transaction {
                    tx: self.tx,
                    client: self.client,
                    amount,
                    kind,
                })
            }
            (TransactionKind::Dispute, TransactionKind::Resolve) => Ok(Transaction {
                tx: self.tx,
                client: self.client,
                amount: self.amount,
                kind,
            }),
            (TransactionKind::Dispute, TransactionKind::ChargeBack) => Ok(Transaction {
                tx: self.tx,
                client: self.client,
                amount: self.amount,
                kind,
            }),
            _ => {
                return Err(TransactionError::InvalidState {
                    from: self.kind,
                    to: kind,
                }
                .into())
            }
        }
    }
}

pub struct MemoryRepo {
    data: RefCell<HashMap<u32, Transaction>>,
}

impl MemoryRepo {
    pub fn new() -> MemoryRepo {
        MemoryRepo {
            data: RefCell::new(HashMap::new()),
        }
    }
    pub fn get_by_client(&self, client: u16) -> Vec<Transaction> {
        self.data
            .borrow()
            .iter()
            .map(|(_, transaction)| transaction)
            .filter(|transaction| transaction.client == client)
            .cloned()
            .collect()
    }
}

pub trait TransactionsRepo {
    fn get(&self, id: u32) -> Result<Option<Transaction>>;
    fn save(&self, transaction: Transaction) -> Result<u32>;
}

impl TransactionsRepo for MemoryRepo {
    fn get(&self, id: u32) -> Result<Option<Transaction>> {
        Ok(self.data.borrow().get(&id).cloned())
    }
    fn save(&self, transaction: Transaction) -> Result<u32> {
        self.data.borrow_mut().insert(transaction.tx, transaction);
        Ok(transaction.tx)
    }
}
