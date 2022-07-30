use anyhow::Result;
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::accounts::Account;
use crate::transactions::{Transaction, TransactionCommand, TransactionsRepo};

pub struct PaymentsEngine<'a> {
    transactions: &'a dyn TransactionsRepo,
    // @TODO: use a trait for the accounts repo
    pub accounts: HashMap<u16, Account>,
}

impl<'a> PaymentsEngine<'a> {
    pub fn new(transactions: &dyn TransactionsRepo) -> PaymentsEngine {
        PaymentsEngine {
            transactions,
            accounts: HashMap::new(),
        }
    }
    pub fn process_transaction(&mut self, t: TransactionCommand) -> Result<()> {
        let transaction = match self.transactions.get(t.tx)? {
            Some(prev) => prev.apply(t.kind)?,
            None => Transaction::try_from(t)?,
        };
        let acc = self
            .accounts
            .entry(transaction.client)
            .or_insert(Account::new(transaction.client));
        acc.apply(transaction)?;
        self.transactions.save(transaction)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::transactions::{MemoryRepo, TransactionKind};
    use rust_decimal::prelude::*;

    use super::*;

    #[test]
    fn test_process() -> Result<()> {
        let repo = MemoryRepo::new();
        let mut engine = PaymentsEngine::new(&repo);
        let amount = Decimal::from(99);
        let command = TransactionCommand {
            kind: TransactionKind::Deposit { amount },
            tx: 1,
            client: 1,
        };
        engine.process_transaction(command.clone())?;
        Ok(())
    }
}
