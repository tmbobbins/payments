/*
* TODO: Split error into separate errors per type or into account specific errors
 */
#[derive(Debug, PartialEq)]
pub enum TransactionErrorKind {
    IncorrectClient,
    Locked,
    AlreadyExists,
    DoesNotExist,
    NegativeBalance,
    AlreadyDisputed,
    FraudulentDispute,
    NotDisputed,
    NoWithdrawalDisputes,
    MustHaveAmount,
    Misc,
}

#[derive(Debug)]
pub struct TransactionError {
    kind: TransactionErrorKind,
    message: String,
}

impl From<&str> for TransactionError {
    fn from(error: &str) -> Self {
        Self {
            kind: TransactionErrorKind::Misc,
            message: error.to_owned(),
        }
    }
}

impl TransactionError {
    pub fn kind(&self) -> &TransactionErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn new(kind: TransactionErrorKind, message: &str) -> Self {
        Self {
            kind,
            message: message.to_string(),
        }
    }

    pub fn incorrect_client() -> Self {
        TransactionError::new(
            TransactionErrorKind::IncorrectClient,
            "Trying to allocate a transaction to the wrong client account",
        )
    }

    pub fn locked() -> Self {
        TransactionError::new(
            TransactionErrorKind::Locked,
            "Account is locked, unable to process further transactions",
        )
    }

    pub fn already_exists() -> Self {
        TransactionError::new(
            TransactionErrorKind::AlreadyExists,
            "Transaction id already exists on account",
        )
    }

    pub fn does_not_exist() -> Self {
        TransactionError::new(
            TransactionErrorKind::DoesNotExist,
            "Transaction id does not exist on account",
        )
    }

    pub fn negative_balance() -> Self {
        TransactionError::new(
            TransactionErrorKind::NegativeBalance,
            "Amount is greater than available, withdrawal would lead to negative balance",
        )
    }

    pub fn already_disputed() -> Self {
        TransactionError::new(
            TransactionErrorKind::AlreadyDisputed,
            "Cannot dispute a transaction that is already in dispute",
        )
    }

    pub fn fraudulent_dispute() -> Self {
        TransactionError::new(
            TransactionErrorKind::FraudulentDispute,
            "Cannot claim a dispute greater than the total balance of the account",
        )
    }

    pub fn not_disputed() -> Self {
        TransactionError::new(
            TransactionErrorKind::NotDisputed,
            "Transaction is not within a disputed state",
        )
    }

    pub fn no_withdrawal_disputes() -> Self {
        TransactionError::new(
            TransactionErrorKind::NoWithdrawalDisputes,
            "Withdrawals are not allowed to be disputed",
        )
    }

    pub fn must_have_amount() -> Self {
        TransactionError::new(
            TransactionErrorKind::MustHaveAmount,
            "Deposits and Withdrawals must have amounts",
        )
    }
}
