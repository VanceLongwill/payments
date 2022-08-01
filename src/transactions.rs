use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;

use anyhow::{anyhow, Result};
use rust_decimal::prelude::*;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TransactionError {
    #[error("unable to move transaction from {from:?} to {to:?}")]
    InvalidState {
        from: TransactionKind,
        to: TransactionKind,
    },
    #[error("unable to apply transaction belonging to a different client: expected {expected:?} got {got:?}")]
    UnexpectedClient { expected: u16, got: u16 },
    #[error(
        "unable to apply transaction with mismatching tx id: expected {expected:?} got {got:?}"
    )]
    UnexpectedTx { expected: u32, got: u32 },
}

/// TransactionCommand represents the minimum fields required for a transaction to be processed.
/// Transaction-kind specific fields are stored withing the TransactionKind enum (e.g. amount for
/// deposits and withdrawals).
#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
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
            _ => Err(anyhow!(
                "transactions must start with a deposit or withdrawal"
            )),
        }
    }
}

/// TransactionKind represents the type of a transaction, including any specific fields that may
/// relate to that particular transaction type.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum TransactionKind {
    Deposit { amount: Decimal },
    Withdrawal { amount: Decimal },
    Dispute,
    Resolve,
    ChargeBack,
}

/// Transaction represents a valid, processed transaction event. A transaction always has a valid amount.
/// For advanced transactions (disputes, resolves, chargebacks), the amount is taken from the
/// transaction which the advanced transaction acts upon.
#[derive(Debug, Clone, Copy)]
pub struct Transaction {
    pub tx: u32,
    pub amount: Decimal,
    pub kind: TransactionKind,
    pub client: u16,
}

impl Transaction {
    pub fn apply(
        &self,
        TransactionCommand { client, kind, tx }: TransactionCommand,
    ) -> Result<Transaction, TransactionError> {
        if self.tx != tx {
            return Err(TransactionError::UnexpectedTx {
                expected: self.tx,
                got: tx,
            });
        }
        if self.client != client {
            return Err(TransactionError::UnexpectedClient {
                expected: self.client,
                got: client,
            });
        }
        // using enums to match only the valid state transitions for a transaction
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
            _ => Err(TransactionError::InvalidState {
                from: self.kind,
                to: kind,
            }),
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
    /// Retrieves the list of transactions for a given client
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
    /// Gets a single transaction by ID
    fn get(&self, id: u32) -> Result<Option<Transaction>> {
        Ok(self.data.borrow().get(&id).cloned())
    }
    /// Upserts a transaction
    fn save(&self, transaction: Transaction) -> Result<u32> {
        self.data.borrow_mut().insert(transaction.tx, transaction);
        Ok(transaction.tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_mismatch() -> Result<()> {
        let transaction = Transaction {
            tx: 1,
            kind: TransactionKind::Withdrawal {
                amount: Decimal::from(8),
            },
            client: 1,
            amount: Decimal::from(8),
        };

        let tx = transaction.tx + 1;
        let res = transaction.apply(TransactionCommand {
            tx,
            kind: TransactionKind::Dispute,
            client: transaction.client,
        });
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            TransactionError::UnexpectedTx {
                expected: transaction.tx,
                got: tx,
            }
        );
        Ok(())
    }

    #[test]
    fn test_client_mismatch() -> Result<()> {
        let transaction = Transaction {
            tx: 1,
            kind: TransactionKind::Withdrawal {
                amount: Decimal::from(8),
            },
            client: 1,
            amount: Decimal::from(8),
        };

        let client = transaction.client + 1;
        let res = transaction.apply(TransactionCommand {
            client,
            kind: TransactionKind::Dispute,
            tx: transaction.tx,
        });
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            TransactionError::UnexpectedClient {
                expected: transaction.client,
                got: client,
            }
        );
        Ok(())
    }

    #[test]
    fn test_apply_valid() -> Result<()> {
        let amount = Decimal::from(100);
        let cases = vec![
            (
                "deposit -> dispute",
                TransactionKind::Deposit { amount },
                TransactionKind::Dispute,
            ),
            (
                "withdrawal -> dispute",
                TransactionKind::Withdrawal { amount },
                TransactionKind::Dispute,
            ),
            (
                "dispute -> resolve",
                TransactionKind::Dispute,
                TransactionKind::Resolve,
            ),
            (
                "dispute -> chargeback",
                TransactionKind::Dispute,
                TransactionKind::ChargeBack,
            ),
        ];

        for (name, from, to) in cases {
            let transaction = Transaction {
                tx: 1,
                kind: from,
                client: 1,
                amount,
            };
            let res = transaction.apply(TransactionCommand {
                kind: to,
                tx: transaction.tx,
                client: transaction.client,
            });
            assert!(res.is_ok(), "{}", name);
            assert_eq!(res.unwrap().kind, to)
        }

        Ok(())
    }

    #[test]
    fn test_apply_invalid() -> Result<()> {
        let amount = Decimal::from(100);
        let cases = vec![
            (
                "deposit -> deposit",
                TransactionKind::Deposit { amount },
                TransactionKind::Deposit { amount },
            ),
            (
                "deposit -> resolve",
                TransactionKind::Deposit { amount },
                TransactionKind::Resolve,
            ),
            (
                "deposit -> chargeback",
                TransactionKind::Deposit { amount },
                TransactionKind::ChargeBack,
            ),
            (
                "withdrawal -> withdrawal",
                TransactionKind::Withdrawal { amount },
                TransactionKind::Withdrawal { amount },
            ),
            (
                "withdrawal -> resolve",
                TransactionKind::Withdrawal { amount },
                TransactionKind::Resolve,
            ),
            (
                "withdrawal -> chargeback",
                TransactionKind::Withdrawal { amount },
                TransactionKind::ChargeBack,
            ),
            (
                "dispute -> dispute",
                TransactionKind::Dispute,
                TransactionKind::Dispute,
            ),
            (
                "dispute -> deposit",
                TransactionKind::Dispute,
                TransactionKind::Deposit { amount },
            ),
            (
                "dispute -> withdrawal",
                TransactionKind::Dispute,
                TransactionKind::Withdrawal { amount },
            ),
            (
                "chargeback -> chargeback",
                TransactionKind::ChargeBack,
                TransactionKind::ChargeBack,
            ),
            (
                "chargeback -> deposit",
                TransactionKind::ChargeBack,
                TransactionKind::Deposit { amount },
            ),
            (
                "chargeback -> withdrawal",
                TransactionKind::ChargeBack,
                TransactionKind::Withdrawal { amount },
            ),
            (
                "chargeback -> dispute",
                TransactionKind::ChargeBack,
                TransactionKind::Dispute,
            ),
            (
                "chargeback -> resolve",
                TransactionKind::ChargeBack,
                TransactionKind::Resolve,
            ),
        ];

        for (name, from, to) in cases {
            let transaction = Transaction {
                tx: 1,
                kind: from,
                client: 1,
                amount,
            };
            let res = transaction.apply(TransactionCommand {
                kind: to,
                tx: transaction.tx,
                client: transaction.client,
            });
            assert!(res.is_err(), "{}", name);
            assert_eq!(
                res.unwrap_err(),
                TransactionError::InvalidState { from, to },
                "{}",
                name
            );
        }

        Ok(())
    }
}
