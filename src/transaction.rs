use rust_decimal::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(flatten)]
    kind: TransactionKind,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum TransactionKind {
    Deposit {
        client: u16,
        tx: u32,
        amount: Decimal,
    },
    Withdrawal {
        client: u16,
        tx: u32,
        amount: Decimal,
    },
    Dispute {
        client: u16,
        tx: u32,
    },
    Resolve {
        client: u16,
        tx: u32,
    },
    ChargeBack {
        client: u16,
        tx: u32,
    },
}
