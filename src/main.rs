use csv::{Reader, ReaderBuilder, Trim, Writer};
use payments::bank::Bank;
use payments::transaction::Transaction;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::{env, io};

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expect 1 argument, but found no arguments")),
        Some(file_path) => Ok(file_path),
    }
}

fn trimmed_csv_reader(file: File) -> Reader<File> {
    let mut builder = ReaderBuilder::new();
    builder.trim(Trim::All);
    builder.from_reader(file)
}

fn add_records_to_bank<T>(mut bank: Bank, mut reader: Reader<T>) -> Result<Bank, Box<dyn Error>>
where
    T: io::Read,
{
    for transaction in reader.deserialize() {
        bank.transact(match transaction {
            Ok(t) => t,
            Err(_) => continue, //TODO: deserialization failure, write to log
        })
    }

    Ok(bank)
}

fn output_bank(bank: Bank) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_writer(io::stdout());
    for account in bank.accounts().values() {
        writer.serialize(account)?;
    }
    writer.flush()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    output_bank(add_records_to_bank(
        Bank::new(),
        trimmed_csv_reader(File::open(get_first_arg()?)?),
    )?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::prelude::Zero;
    use rust_decimal::Decimal;
    use std::fs::remove_file;
    use std::ops::Mul;
    use std::str::FromStr;

    fn decimal_str(decimal: &str) -> Decimal {
        Decimal::from_str(decimal).unwrap()
    }

    fn file_writer(file_name: &str) -> Writer<File> {
        Writer::from_writer(File::create(file_name).unwrap())
    }

    fn delete_file(file_name: &str) {
        remove_file(file_name).unwrap();
    }

    fn add_deposits(
        writer: &mut Writer<File>,
        client_id: u16,
        value: Decimal,
        amount: u32,
        offset: u32,
    ) {
        for id in offset..offset + amount {
            writer
                .serialize(Transaction::new_deposit(client_id, id, value))
                .unwrap();
        }
        writer.flush().unwrap()
    }

    #[test]
    fn test_new() {
        let file_name = "integration_test.csv";
        let mut writer = file_writer(file_name);
        add_deposits(&mut writer, 1, decimal_str("10.0505"), 200_000, 0);
        add_deposits(&mut writer, 2, decimal_str("20.7836"), 500_000, 200_000);
        add_deposits(&mut writer, 3, decimal_str("25700.1234"), 100_000, 700_000);

        let bank = add_records_to_bank(
            Bank::new(),
            trimmed_csv_reader(File::open(file_name).unwrap()),
        )
        .unwrap();

        let accounts = bank.accounts();
        let account_1 = &accounts[&1];
        let expected = decimal_str("10.0505").mul(decimal_str("200000"));
        assert_eq!(&expected, account_1.available());
        assert_eq!(&Decimal::zero(), account_1.held());
        assert_eq!(&expected, account_1.total());

        let account_2 = &accounts[&2];
        let expected = decimal_str("20.7836").mul(decimal_str("500000"));
        assert_eq!(&expected, account_2.available());
        assert_eq!(&Decimal::zero(), account_2.held());
        assert_eq!(&expected, account_2.total());

        let account_3 = &accounts[&3];
        let expected = decimal_str("25700.1234").mul(decimal_str("100000"));
        assert_eq!(&expected, account_3.available());
        assert_eq!(&Decimal::zero(), account_3.held());
        assert_eq!(&expected, account_3.total());

        delete_file(file_name);
    }
}
