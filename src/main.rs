use serde::{Deserialize};
use std::error::Error;
use std::collections::HashMap;
use std::env;
use std::io::{self};

trait ClientAndId {
    fn id(&self) -> u32;
    fn client_id(&self) -> u16;
}

trait Amount {
    fn amount(&self) -> f64;
}

#[derive(Debug, Deserialize)]
struct TransactionCsvElement {
    pub r#type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>
}

#[derive(Debug)]
struct TransactionBody {
    id: u32,
    client_id: u16,
}

#[derive(Debug)]
struct TransactionBodyWithAmount {
    id: u32,
    client_id: u16,
    amount: f64,
}

#[derive(Debug)]
enum Transaction {
    Deposit(TransactionBodyWithAmount),
    Withdraw(TransactionBodyWithAmount),
    Dispute(TransactionBody),
    Resolve(TransactionBody),
    Chargeback(TransactionBody),
}

impl ClientAndId for TransactionBody {
    fn id(&self) -> u32 {
        self.id
    }

    fn client_id(&self) -> u16 {
        self.client_id
    }
}

impl ClientAndId for TransactionBodyWithAmount {
    fn id(&self) -> u32 {
        self.id
    }

    fn client_id(&self) -> u16 {
        self.client_id
    }
}

impl Amount for TransactionBodyWithAmount {
    fn amount(&self) -> f64 {
        self.amount
    }
}

impl From<TransactionCsvElement> for Transaction {
    fn from(tx: TransactionCsvElement) -> Transaction {
        match tx.r#type.as_str() {
            "deposit"  => Transaction::Deposit(TransactionBodyWithAmount { id: tx.tx, client_id: tx.client, amount: tx.amount.unwrap() }),
            "withdrawal" => Transaction::Withdraw(TransactionBodyWithAmount { id: tx.tx, client_id: tx.client, amount: tx.amount.unwrap() }),
            "dispute" => Transaction::Dispute(TransactionBody{ id: tx.tx, client_id: tx.client }),
            "resolve" => Transaction::Resolve(TransactionBody{ id: tx.tx, client_id: tx.client }),
            "chargeback" => Transaction::Chargeback(TransactionBody{ id: tx.tx, client_id: tx.client }) ,
            _ => panic!("can't parse transaction"),
        }
    }
}

#[derive(Default, Debug)]
struct Account {
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl Account {
    pub fn deposit(&mut self, amount: f64) {
        self.available += amount;
        self.total += amount;
    }

    pub fn withdrawal(&mut self, amount: f64) {
        self.available -= amount;
        self.total-= amount;
    }

    pub fn dispute(&mut self, amount: f64) {
        self.available -= amount;
        self.held += amount;
    }

    pub fn resolve_dispute(&mut self, amount: f64) {
        self.available += amount;
        self.held -= amount;
    }

    pub fn chargeback(&mut self, amount: f64){
        self.held -= amount;
        self.total -= amount;
        self.locked = true
    }
}

#[derive(Default, Debug)]
struct Ledger{
    pub state: HashMap<u16, Account>,
    pub history: HashMap<u32, TransactionBodyWithAmount>,
}

impl Ledger {
    pub fn process_txs(&mut self, txs: Vec<Transaction>) {
        for tx in txs {
            match tx {
                Transaction::Deposit(e) => self.process_deposit(e),
                Transaction::Withdraw(e) => self.process_withdrawal(e),
                Transaction::Dispute(e) => self.process_dispute(e),
                Transaction::Resolve(e) => self.process_resolve(e),
                Transaction::Chargeback(e) => self.process_chargeback(e),
            }
        }
    }

    fn process_deposit(&mut self, tx: TransactionBodyWithAmount) {
        if !self.state.contains_key(&tx.client_id()) {
            self.state.insert(tx.client_id(), Account::default());
        }

        let account = self.state.get_mut(&tx.client_id()).unwrap();
        account.deposit(tx.amount());
        self.history.insert(tx.id(), tx);
    }

    fn process_withdrawal(&mut self, tx: TransactionBodyWithAmount){
        if self.state.contains_key(&tx.client_id()) {
            let account = self.state.get_mut(&tx.client_id()).unwrap();

            if account.available >= tx.amount() {
                account.withdrawal(tx.amount());
            }
        }
    }

    fn process_dispute(&mut self, tx: TransactionBody) {
        if self.state.contains_key(&tx.client_id()) {
            let account = self.state.get_mut(&tx.client_id()).unwrap();
            let disputed_tx = self.history.get(&tx.id());
            match disputed_tx {
                Some(a) => account.dispute(a.amount()),
                None => ()
            }
        }
    }

    fn process_resolve(&mut self, tx: TransactionBody) {
        if self.state.contains_key(&tx.client_id()) {
            let account = self.state.get_mut(&tx.client_id()).unwrap();
            let resolved_tx = self.history.get(&tx.id());
            match resolved_tx {
                Some(a) => account.resolve_dispute(a.amount()),
                None => ()
            }

        }
    }

    fn process_chargeback(&mut self, tx: TransactionBody) {
        if self.state.contains_key(&tx.client_id()) {
            let account = self.state.get_mut(&tx.client_id()).unwrap();
            let chargeback_tx = self.history.get(&tx.id());
            match chargeback_tx {
                Some(a) => if a.amount() <= account.held {
                    account.chargeback(a.amount());
                },
                None => ()
            }
        }
    }
}

fn parse_csv(csv_name: &str) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(&csv_name)?;
    let mut all_transactions: Vec<Transaction> = Vec::new();
    for result in rdr.deserialize() {
        let record: TransactionCsvElement = result?;
        all_transactions.push(Transaction::from(record));
    }
    Ok(all_transactions)
}

fn write_to_stdout(ledger: HashMap<u16, Account>) -> Result<(), Box<dyn Error>> {
    let mut writer = csv::Writer::from_writer(io::stdout());
    writer.write_record(&["client","available","held","total","locked"])?;

    for (key, value) in ledger.iter() {
        writer.write_record(&[&key.to_string(), &value.available.to_string(),
            &value.held.to_string(), &value.total.to_string(), &value.locked.to_string()
        ]);
    }
    writer.flush()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename: &String = &args[1];
    let transactions: Vec<Transaction> = parse_csv(filename).unwrap();
    let mut ledger = Ledger::default();
    ledger.process_txs(transactions);
    let processed_tr = ledger.state;
    write_to_stdout(processed_tr);
}