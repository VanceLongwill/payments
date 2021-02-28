use anyhow::{anyhow, Result};
use rust_decimal::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Deserialize)]
pub struct TransactionCommand {
    #[serde(flatten)]
    pub kind: TransactionKind,
    pub tx: u32,
    pub client: u16,
}

#[derive(Debug, Clone, Copy, Deserialize)]
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

impl Transaction {
    pub fn next(&self, kind: TransactionKind) -> Result<Transaction> {
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
                return Err(anyhow!(
                    "unable to move transaction from {:?} to {:?}",
                    self.kind,
                    kind
                ))
            }
        }
    }
}

pub struct Repo {
    data: HashMap<u32, Transaction>,
}

impl Repo {
    pub fn new() -> Repo {
        Repo {
            data: HashMap::new(),
        }
    }
    pub fn get(&self, id: u32) -> Option<&Transaction> {
        self.data.get(&id)
    }
    pub fn get_by_client(&self, client: u16) -> Vec<&Transaction> {
        self.data
            .iter()
            .map(|(_, transaction)| transaction)
            .filter(|transaction| transaction.client == client)
            .collect()
    }
    pub fn save(&mut self, transaction: Transaction) -> u32 {
        self.data.insert(transaction.tx, transaction);
        transaction.tx
    }
}
