use std::{collections::HashMap, convert::TryFrom};
use anyhow::Result;

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

