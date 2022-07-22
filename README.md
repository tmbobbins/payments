# Payments
Simple payments engine

## Prerequisites
- Rust toolkit (1.59.0)

## Usage
```shell
cargo run -- test.csv > output.csv
```

## Assumptions
- frozen & locked are synonymous
- any level of dispute must be done by the same client and thus only affect the balance of the client that owns the original transaction
- disputes can only be enacted upon deposits (this was assumed based on the wording of dispute, resolve, chargeback)
- never allow for negative balance
- once resolved a deposit transaction can be disputed again

## Open questions
- Time value of money considerations on disputes
- forex considerations
- Jurisdictional considerations on disputes
- account unlocking

## Correctness
Correctness is partially validated through account and transaction unit tests,
type system (with Serde) as well as manual file testing.
Automated integration testing as well as more thorough unit testing is desired.

## Next steps
- implement streaming mechanism for file reading, to avoid OOM issues on high volume / resource constrained environments
- implement per client work pools (threads) (potentially with actor model)
- restructure to have per transaction type traits (Deposit, Withdrawal, Dispute, Resolve, Chargeback)
- explore data storage for transactional persistence (redis / postgres / etc...)
- Although there's a number of tests around the transactions in the account the project is lacking testing in some areas and also lacking any automated integration tests
- Handle withdrawal disputes, fundamentally inverting the logic of dispute, resolve & chargeback
- Add in error export to file / sentry / etc.
- enforce 4 dp across implementation (current handles up to 28 (rust_decimal))

## Dependencies
#### ahash
[![dependency status](https://deps.rs/crate/ahash/0.7.6/status.svg)](https://deps.rs/crate/ahash/0.7.6)
#### csv
[![dependency status](https://deps.rs/crate/csv/1.1.6/status.svg)](https://deps.rs/crate/csv/1.1.6)
#### serde
[![dependency status](https://deps.rs/crate/serde/1.0.140/status.svg)](https://deps.rs/crate/serde/1.0.140)
#### rust_decimal
[![dependency status](https://deps.rs/crate/rust_decimal/1.25.0/status.svg)](https://deps.rs/crate/rust_decimal/1.25.0) \
appears to have vulnerability through diesel, however not in use here.