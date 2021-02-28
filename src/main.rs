use anyhow::Result;
use clap::Clap;
use csv;
use rust_decimal::prelude::*;
use serde::Serialize;
use std::error::Error;
use std::io;
use std::{collections::HashMap, convert::TryFrom};

pub mod account;
pub mod transaction;

use account::Account;
use transaction::{Repo, Transaction, TransactionCommand};

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Vance Longwill <vancelongwill@gmail.com>")]
struct Opts {
    file: String,
}

#[derive(Debug, Serialize)]
struct AccountStatement {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

struct PaymentsEngine {
    store: Repo,
    accounts: HashMap<u16, Account>,
}

impl PaymentsEngine {
    fn new() -> PaymentsEngine {
        PaymentsEngine {
            store: Repo::new(),
            accounts: HashMap::new(),
        }
    }
    fn process_transaction(&mut self, t: Transaction) -> Result<()> {
        let transaction = if let Some(prev) = self.store.get(t.tx) {
            prev.next(t.kind)?
        } else {
            t
        };
        let acc = self
            .accounts
            .entry(transaction.client)
            .or_insert(Account::new());
        acc.apply(transaction)?;
        self.store.save(transaction);
        Ok(())
    }
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
