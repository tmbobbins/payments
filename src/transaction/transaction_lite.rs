use crate::transaction::transaction_error::TransactionError;
use crate::transaction::transaction_type::TransactionType;
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct TransactionLite {
    pub disputed: bool,
    pub t_type: TransactionType,
    pub amount: Decimal,
}

impl TransactionLite {
    pub fn new(t_type: TransactionType, amount: Decimal) -> Self {
        Self {
            disputed: false,
            t_type,
            amount,
        }
    }

    pub fn disputed_or_err(&self) -> Result<(), TransactionError> {
        if !self.disputed {
            return Err(TransactionError::not_disputed());
        }

        Ok(())
    }
}
