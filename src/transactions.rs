use anyhow::{anyhow, Result};
use rust_decimal::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

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
