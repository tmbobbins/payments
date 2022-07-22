pub mod transaction_error;
pub mod transaction_lite;
pub mod transaction_type;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use transaction_error::TransactionError;
use transaction_type::TransactionType;
use transaction_type::TransactionType::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};

/*
 * TODO: Move this to trait (transact) with custom deserializer using type and injecting the account
 * thus moving the logic out of the account for each transaction type
 * e.g.
 * pub trait transact { fn transact(account: &mut Account) -> TransactionResult<()> }
 * impl Transact for Deposit {}
 * impl Transact for Withdrawal {}
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub t_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub id: u32,
    pub amount: Option<Decimal>,
}

impl Transaction {
    fn new(t_type: TransactionType, client_id: u16, id: u32, amount: Option<Decimal>) -> Self {
        Self {
            t_type,
            client_id,
            id,
            amount,
        }
    }

    pub fn new_deposit(client_id: u16, id: u32, amount: Decimal) -> Self {
        Transaction::new(Deposit, client_id, id, Some(amount))
    }

    pub fn new_withdrawal(client_id: u16, id: u32, amount: Decimal) -> Self {
        Transaction::new(Withdrawal, client_id, id, Some(amount))
    }

    pub fn new_dispute(client_id: u16, id: u32) -> Self {
        Transaction::new(Dispute, client_id, id, None)
    }

    pub fn new_resolve(client_id: u16, id: u32) -> Self {
        Transaction::new(Resolve, client_id, id, None)
    }

    pub fn new_chargeback(client_id: u16, id: u32) -> Self {
        Transaction::new(Chargeback, client_id, id, None)
    }

    pub fn amount(&self) -> Result<Decimal, TransactionError> {
        self.amount.ok_or_else(TransactionError::must_have_amount)
    }
}
