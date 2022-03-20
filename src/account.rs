use crate::transaction::transaction_error::TransactionError;
use crate::transaction::transaction_lite::TransactionLite;
use crate::transaction::transaction_type::TransactionType;
use crate::transaction::Transaction;
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub type TransactionResult<T> = Result<T, TransactionError>;

pub fn decimal_normalize_serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    rust_decimal::serde::str::serialize(&value.normalize(), serializer)
}

#[derive(Serialize, Debug)]
pub struct Account {
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(skip)]
    transactions: HashMap<u32, TransactionLite>,
    #[serde(serialize_with = "decimal_normalize_serialize")]
    available: Decimal,
    #[serde(serialize_with = "decimal_normalize_serialize")]
    held: Decimal,
    #[serde(serialize_with = "decimal_normalize_serialize")]
    total: Decimal,
    locked: bool,
}

impl Account {
    pub fn new(client_id: u16) -> Self {
        Self {
            client_id,
            transactions: HashMap::new(),
            available: Decimal::new(0, 4),
            held: Decimal::new(0, 4),
            total: Decimal::new(0, 4),
            locked: false,
        }
    }

    pub fn transact(&mut self, transaction: Transaction) -> TransactionResult<()> {
        if transaction.client_id != self.client_id {
            return Err(TransactionError::incorrect_client());
        }

        if self.locked {
            return Err(TransactionError::locked());
        }

        match transaction.t_type {
            TransactionType::Deposit => self.deposit(transaction)?,
            TransactionType::Withdrawal => self.withdrawal(transaction)?,
            TransactionType::Dispute => self.dispute(transaction)?,
            TransactionType::Resolve => self.resolve(transaction)?,
            TransactionType::Chargeback => self.chargeback(transaction)?,
        }

        Ok(())
    }

    fn add_to_transactions(&mut self, transaction: &Transaction) -> TransactionResult<()> {
        match self.transactions.entry(transaction.id) {
            Entry::Occupied(_) => return Err(TransactionError::already_exists()),
            Entry::Vacant(transactions) => {
                transactions.insert(TransactionLite::new(
                    transaction.t_type.clone(),
                    transaction.amount()?,
                ));
            }
        }

        Ok(())
    }

    fn deposit(&mut self, transaction: Transaction) -> TransactionResult<()> {
        self.add_to_transactions(&transaction)?;
        let amount = self
            .transactions
            .get_mut(&transaction.id)
            .ok_or_else(TransactionError::does_not_exist)?
            .amount;

        self.total += amount;
        self.available += amount;

        Ok(())
    }

    fn withdrawal(&mut self, transaction: Transaction) -> TransactionResult<()> {
        let amount = transaction.amount()?;
        if amount > self.available {
            return Err(TransactionError::negative_balance());
        }

        self.add_to_transactions(&transaction)?;

        self.available -= amount;
        self.total -= amount;

        Ok(())
    }

    fn dispute(&mut self, transaction: Transaction) -> TransactionResult<()> {
        let mut disputed_transaction = self
            .transactions
            .get_mut(&transaction.id)
            .ok_or_else(TransactionError::does_not_exist)?;

        if disputed_transaction.disputed {
            return Err(TransactionError::already_disputed());
        }

        if disputed_transaction.t_type == TransactionType::Withdrawal {
            return Err(TransactionError::no_withdrawal_disputes());
        }

        let amount = disputed_transaction.amount;
        if amount > self.total {
            return Err(TransactionError::fraudulent_dispute());
        }

        disputed_transaction.disputed = true;
        self.held += amount;
        self.available -= amount;

        Ok(())
    }

    fn get_disputed_transaction(&mut self, id: &u32) -> TransactionResult<&mut TransactionLite> {
        let disputed_transaction = self
            .transactions
            .get_mut(id)
            .ok_or_else(TransactionError::does_not_exist)?;
        disputed_transaction.disputed_or_err()?;

        Ok(disputed_transaction)
    }

    fn resolve(&mut self, transaction: Transaction) -> TransactionResult<()> {
        let disputed_transaction = self.get_disputed_transaction(&transaction.id)?;
        disputed_transaction.disputed = false;

        let amount = disputed_transaction.amount;
        self.held -= amount;
        self.available += amount;

        Ok(())
    }

    fn chargeback(&mut self, transaction: Transaction) -> TransactionResult<()> {
        let disputed_transaction = self.get_disputed_transaction(&transaction.id)?;
        disputed_transaction.disputed = false;

        let amount = disputed_transaction.amount;
        self.held -= amount;
        self.total -= amount;
        self.locked = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    /* TODO: Test multiple values */
    use super::*;
    use crate::transaction::transaction_error::TransactionErrorKind;
    use rust_decimal::prelude::Zero;
    use std::str::FromStr;

    fn decimal_str(decimal: &str) -> Decimal {
        Decimal::from_str(decimal).unwrap()
    }

    #[test]
    fn test_new() {
        let account = Account::new(1);

        assert_eq!(Decimal::zero(), account.available);
        assert_eq!(Decimal::zero(), account.total);
        assert_eq!(Decimal::zero(), account.held);
        assert!(!account.locked);
    }

    #[test]
    fn test_deposit() {
        let deposit_value = decimal_str("1");
        let mut account = Account::new(1);
        account
            .transact(Transaction::new_deposit(1, 1, deposit_value))
            .unwrap();

        assert_eq!(deposit_value, account.available);
        assert_eq!(deposit_value, account.total);
        assert_eq!(Decimal::zero(), account.held);
        assert!(!account.locked);
    }

    #[test]
    fn test_valid_withdrawal() {
        let deposit_value = decimal_str("2");
        let mut account = Account::new(1);
        account
            .transact(Transaction::new_deposit(1, 1, deposit_value))
            .unwrap();

        let withdrawal_value = decimal_str("1");
        account
            .transact(Transaction::new_withdrawal(1, 2, withdrawal_value))
            .unwrap();

        assert_eq!(deposit_value - withdrawal_value, account.available);
        assert_eq!(deposit_value - withdrawal_value, account.total);
        assert_eq!(Decimal::zero(), account.held);
        assert!(!account.locked);

        account
            .transact(Transaction::new_withdrawal(1, 3, withdrawal_value))
            .unwrap();

        assert_eq!(Decimal::zero(), account.available);
        assert_eq!(Decimal::zero(), account.total);
        assert_eq!(Decimal::zero(), account.held);
        assert!(!account.locked);
    }

    #[test]
    fn test_negative_balance_withdrawal() {
        let deposit_value = decimal_str("2");
        let mut account = Account::new(1);
        account
            .transact(Transaction::new_deposit(1, 1, deposit_value))
            .unwrap();

        let withdrawal_value = decimal_str("3");
        let err = account
            .transact(Transaction::new_withdrawal(1, 2, withdrawal_value))
            .unwrap_err();

        assert_eq!(&TransactionErrorKind::NegativeBalance, err.kind());
        assert_eq!(deposit_value, account.available);
        assert_eq!(deposit_value, account.total);
        assert_eq!(Decimal::zero(), account.held);
        assert!(!account.locked);
    }

    #[test]
    fn test_resolve_dispute() {
        let deposit_value = decimal_str("2");
        let mut account = Account::new(1);
        account
            .transact(Transaction::new_deposit(1, 1, deposit_value))
            .unwrap();
        account.transact(Transaction::new_dispute(1, 1)).unwrap();

        assert_eq!(Decimal::zero(), account.available);
        assert_eq!(deposit_value, account.total);
        assert_eq!(deposit_value, account.held);
        assert!(!account.locked);

        account.transact(Transaction::new_resolve(1, 1)).unwrap();

        assert_eq!(false, account.transactions.get(&1).unwrap().disputed);
        assert_eq!(deposit_value, account.available);
        assert_eq!(deposit_value, account.total);
        assert_eq!(Decimal::zero(), account.held);
        assert!(!account.locked);
    }

    #[test]
    fn test_chargeback() {
        let deposit_value = decimal_str("2");
        let mut account = Account::new(1);
        account
            .transact(Transaction::new_deposit(1, 1, deposit_value))
            .unwrap();
        account.transact(Transaction::new_dispute(1, 1)).unwrap();
        account.transact(Transaction::new_chargeback(1, 1)).unwrap();

        assert_eq!(Decimal::zero(), account.available);
        assert_eq!(Decimal::zero(), account.total);
        assert_eq!(Decimal::zero(), account.held);
        assert!(account.locked);
    }

    #[test]
    fn test_already_disputed_dispute() {
        let deposit_value = decimal_str("2");
        let mut account = Account::new(1);
        account
            .transact(Transaction::new_deposit(1, 1, deposit_value))
            .unwrap();
        account.transact(Transaction::new_dispute(1, 1)).unwrap();
        let err = account
            .transact(Transaction::new_dispute(1, 1))
            .unwrap_err();

        assert_eq!(&TransactionErrorKind::AlreadyDisputed, err.kind());
        assert_eq!(Decimal::zero(), account.available);
        assert_eq!(deposit_value, account.total);
        assert_eq!(deposit_value, account.held);
        assert!(!account.locked);
    }

    #[test]
    fn test_withdrawal_dispute_failure() {
        let deposit_value = decimal_str("2");
        let withdrawal_value = decimal_str("1");
        let mut account = Account::new(1);
        account
            .transact(Transaction::new_deposit(1, 1, deposit_value))
            .unwrap();
        account
            .transact(Transaction::new_withdrawal(1, 2, withdrawal_value))
            .unwrap();
        let err = account
            .transact(Transaction::new_dispute(1, 2))
            .unwrap_err();

        assert_eq!(&TransactionErrorKind::NoWithdrawalDisputes, err.kind());
        assert_eq!(deposit_value - withdrawal_value, account.available);
        assert_eq!(deposit_value - withdrawal_value, account.total);
        assert_eq!(Decimal::zero(), account.held);
        assert!(!account.locked);
    }

    #[test]
    fn test_undisputed_resolution_failure() {
        let deposit_value = decimal_str("2");
        let mut account = Account::new(1);
        account
            .transact(Transaction::new_deposit(1, 1, deposit_value))
            .unwrap();
        let err = account
            .transact(Transaction::new_chargeback(1, 1))
            .unwrap_err();
        assert_eq!(&TransactionErrorKind::NotDisputed, err.kind());

        let err = account
            .transact(Transaction::new_resolve(1, 1))
            .unwrap_err();
        assert_eq!(&TransactionErrorKind::NotDisputed, err.kind());

        assert_eq!(deposit_value, account.available);
        assert_eq!(deposit_value, account.total);
        assert_eq!(Decimal::zero(), account.held);
        assert!(!account.locked);
    }

    #[test]
    fn test_fraudulent_dispute() {
        let deposit_value = decimal_str("2");
        let withdrawal_value = decimal_str("1");
        let mut account = Account::new(1);
        account
            .transact(Transaction::new_deposit(1, 1, deposit_value))
            .unwrap();
        account
            .transact(Transaction::new_withdrawal(1, 2, withdrawal_value))
            .unwrap();
        let err = account
            .transact(Transaction::new_dispute(1, 1))
            .unwrap_err();

        assert_eq!(&TransactionErrorKind::FraudulentDispute, err.kind());
        assert_eq!(deposit_value - withdrawal_value, account.available);
        assert_eq!(deposit_value - withdrawal_value, account.total);
        assert_eq!(Decimal::zero(), account.held);
        assert!(!account.locked); // TODO: we would most likely want to lock an account pending investigation here?
    }

    #[test]
    fn test_transaction_does_not_exist() {
        let mut account = Account::new(1);
        let err = account
            .transact(Transaction::new_dispute(1, 1))
            .unwrap_err();
        assert_eq!(&TransactionErrorKind::DoesNotExist, err.kind());

        let err = account
            .transact(Transaction::new_resolve(1, 1))
            .unwrap_err();
        assert_eq!(&TransactionErrorKind::DoesNotExist, err.kind());

        let err = account
            .transact(Transaction::new_chargeback(1, 1))
            .unwrap_err();
        assert_eq!(&TransactionErrorKind::DoesNotExist, err.kind());

        assert_eq!(Decimal::zero(), account.available);
        assert_eq!(Decimal::zero(), account.total);
        assert_eq!(Decimal::zero(), account.held);
        assert!(!account.locked);
    }
}
