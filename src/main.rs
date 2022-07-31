extern crate proc_macro;

use anyhow::Result;
use clap::Clap;
use csv;
use rust_decimal::prelude::*;
use serde::Serialize;
use std::io;
use tracing::{debug, error};
use tracing_subscriber;

mod accounts;
mod payments;
mod transactions;

use payments::PaymentsEngine;
use transactions::MemoryRepo;

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

fn run() -> Result<()> {
    let opts: Opts = Opts::parse();

    let mut reader = csv::Reader::from_path(opts.file)?;
    let repo = MemoryRepo::new();
    let mut engine = PaymentsEngine::new(&repo);

    for result in reader.deserialize() {
        let command = result?;
        match engine.process_transaction(command) {
            Ok(()) => debug!(
                tx = command.tx,
                client = command.client,
                "Processed transaction"
            ),
            Err(e) => debug!(
                error = e.to_string(),
                tx = command.tx,
                client = command.client,
                "Unable to process transaction"
            ),
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
    tracing_subscriber::fmt::init();

    if let Err(e) = run() {
        error!(error = e.to_string(), "Something went wrong")
    }
}
