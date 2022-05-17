use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::error::Error;
use std::io;
use std::process;


#[derive(Debug, Deserialize)]
struct Transaction {
    action: String,
    client_id: u16,
    tx_id: u32,
    amount: Option<f32>
}

enum Action {
    Increment {amount: f32},
    Decrement {amount: f32},
    Dispute {amount: Option<f32>},
    Resolve {amount: Option<f32>},
    Chargeback {amount: Option<f32>}
}

// parse csv ->
// create clients with transactions ->
// pass it to calculator ->
// calculate for each client and write to csv
fn parse_csv() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path("tests/test.csv")?;
    let mut groups: Vec<Transaction> = Vec::new();
    for result in rdr.deserialize() {
        let record: Transaction = result?;
        groups.push(record);
        //println!("{:?}", record);
        //println!("{:?}", record.action);
    }
    let sorted = groups.into_iter().map(|x| x.client_id).collect();


    println!("{:?}", sorted);
    Ok(())
}

fn main() {
    if let Err(err) = parse_csv() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
