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



fn parse_csv() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path("tests/test.csv")?;
    for result in rdr.deserialize() {
        let record: Transaction = result?;
        let x = match record.client_id {
            1 => println!("pog"),
            _ => {}
        };

        // parse csv ->
        // create clients with transactions ->
        // pass it to calculator ->
        // calculate for each client and write to csv

        println!("{:?}", x);
        //println!("{:?}", record.action);
    }
    Ok(())
}

fn main() {
    if let Err(err) = parse_csv() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
