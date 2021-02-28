use anyhow::Result;
use std::collections::HashMap;

use crate::accounts::Account;
use crate::transactions::{Repo, Transaction};

pub struct PaymentsEngine {
    store: Repo,
    pub accounts: HashMap<u16, Account>,
}

impl PaymentsEngine {
    pub fn new() -> PaymentsEngine {
        PaymentsEngine {
            store: Repo::new(),
            accounts: HashMap::new(),
        }
    }
    pub fn process_transaction(&mut self, t: Transaction) -> Result<()> {
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
