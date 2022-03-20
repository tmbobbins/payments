use csv::{Reader, ReaderBuilder, Trim, Writer};
use payments::bank::Bank;
use payments::transaction::Transaction;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::{env, io};

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expect 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

fn trimmed_csv_reader(file: File) -> Reader<File> {
    let mut builder = ReaderBuilder::new();
    builder.trim(Trim::All);
    builder.from_reader(file)
}

fn add_records_to_bank(mut bank: Bank, file: File) -> Result<Bank, Box<dyn Error>> {
    for transaction in trimmed_csv_reader(file).deserialize() {
        let transaction: Transaction = match transaction {
            Ok(t) => t,
            Err(_) => continue, //TODO: deserialization failure, write to log
        };
        bank.transact(transaction)
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
        File::open(get_first_arg()?)?,
    )?)?;

    Ok(())
}
