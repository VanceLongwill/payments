use anyhow::{anyhow, Result};
use clap::Clap;
use csv;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::io;

use payments::payments::PaymentsEngine;
use payments::transactions::{Transaction, TransactionKind};

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Vance Longwill <vancelongwill@gmail.com>")]
struct Opts {
    file: String,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Serialize)]
struct AccountStatement {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

fn run() -> Result<()> {
    let opts: Opts = Opts::parse();
    let mut reader = csv::Reader::from_path(opts.file)?;
    let mut engine = PaymentsEngine::new();
    for result in reader.deserialize() {
        let command: TransactionCommand = result?;
        let transaction = Transaction::try_from(command)?;
        if let Err(_e) = engine.process_transaction(transaction) {
            // println!("Unable to process transaction: {:?}", e);
            // println!("{:?}", transaction);
        }
    }

    let mut writer = csv::Writer::from_writer(io::stdout());
    for (client, acc) in engine.accounts.into_iter() {
        writer.serialize(AccountStatement {
            client,
            available: acc.available(),
            held: acc.held(),
            total: acc.total(),
            locked: acc.is_locked(),
        })?;
    }
    writer.flush()?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Something went wrong: {:?}", e)
    }
}
