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

use accounts::{AccountsRepo, MemoryRepo as AccountsMemoryRepo};
use payments::PaymentsEngine;
use transactions::MemoryRepo as TransactionsMemoryRepo;

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
    // @TODO: as we scale, the in-memory repositories might no longer be suitable due to
    // memory constraints & cold start (loading all transactions that ever occurred into memory
    // from CSV vs snapshotting the state at a known point in time).
    //
    // To mitigate this, the in-memory implementations can be easily swapped out for ones
    // utilising a db with a higher capacity & more durable storage backend (e.g. sqlite, redis, postgres or dynamodb).
    // Migrating to one of the above storage backends is as simple as implementing the
    // AccountsRepo/TransactionsRepo traits respectively.
    let transactions_repo = TransactionsMemoryRepo::new();
    let accounts_repo = AccountsMemoryRepo::new();
    let mut engine = PaymentsEngine::new(&transactions_repo, &accounts_repo);

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
    for acc in accounts_repo.get_all()? {
        writer.serialize(AccountStatement {
            client: acc.client(),
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
