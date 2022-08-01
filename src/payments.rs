use anyhow::Result;
use std::convert::TryFrom;

use crate::accounts::{Account, AccountsRepo};
use crate::transactions::{Transaction, TransactionCommand, TransactionsRepo};

pub struct PaymentsEngine<'a, 'b> {
    transactions: &'a dyn TransactionsRepo,
    accounts: &'b dyn AccountsRepo,
}

impl<'a, 'b> PaymentsEngine<'a, 'b> {
    pub fn new(
        transactions: &'a dyn TransactionsRepo,
        accounts: &'b dyn AccountsRepo,
    ) -> PaymentsEngine<'a, 'b> {
        PaymentsEngine {
            transactions,
            accounts,
        }
    }
    /// process_transaction attempts to create a transaction event and apply that transaction to
    /// the client account it references
    pub fn process_transaction(&self, t: TransactionCommand) -> Result<()> {
        let transaction = match self.transactions.get(t.tx)? {
            Some(prev) => prev.apply(t)?,
            None => Transaction::try_from(t)?,
        };

        let updated = match self.accounts.get(transaction.client)? {
            Some(acc) => acc.apply(transaction)?,
            None => Account::new(transaction)?,
        };

        self.accounts.save(updated)?;
        self.transactions.save(transaction)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::accounts::MemoryRepo as AccountsMemoryRepo;
    use crate::transactions::{MemoryRepo as TransactionsMemoryRepo, TransactionKind};
    use rust_decimal::prelude::*;

    use super::*;

    #[test]
    fn test_process() -> Result<()> {
        let transactions_repo = TransactionsMemoryRepo::new();
        let accounts_repo = AccountsMemoryRepo::new();
        let engine = PaymentsEngine::new(&transactions_repo, &accounts_repo);
        let amount = Decimal::from(99);
        let command = TransactionCommand {
            kind: TransactionKind::Deposit { amount },
            tx: 1,
            client: 1,
        };
        engine.process_transaction(command)?;
        Ok(())
    }
}
