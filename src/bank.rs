use crate::account::Account;
use crate::transaction::Transaction;
use std::collections::HashMap;

#[derive(Default)]
pub struct Bank {
    accounts: HashMap<u16, Account>,
    transactions: HashMap<u32, u16>,
}

impl Bank {
    pub fn new() -> Self {
        Bank::default()
    }

    pub fn accounts(&self) -> &HashMap<u16, Account> {
        &self.accounts
    }

    fn get_or_insert_client_id(&mut self, transaction: &Transaction) -> u16 {
        *self
            .transactions
            .entry(transaction.id)
            .or_insert_with(|| transaction.client_id)
    }
    fn duplicate_transaction_id(&mut self, transaction: &Transaction) -> bool {
        self.get_or_insert_client_id(transaction) != transaction.client_id
    }

    pub fn transact(&mut self, transaction: Transaction) {
        /*
         *   TODO: ignore result, should most likely implement a file system log here to output
         *   any errors, rather than ignoring them as STDOUT is used to output the csv
         */
        if self.duplicate_transaction_id(&transaction) {
            return;
        }

        let _ = self
            .accounts
            .entry(transaction.client_id)
            .or_insert_with(|| Account::new(transaction.client_id))
            .transact(transaction);
    }
}
