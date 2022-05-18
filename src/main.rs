use serde::{Serialize, Deserialize};
use std::error::Error;
use std::io;
use std::process;
use std::collections::HashMap;


trait ClientAndId {
    fn id(&self) -> u32;
    fn client_id(&self) -> u16;
}

trait Amount {
    fn amount(&self) -> f64;
}

#[derive(Debug)]
struct TransactionBody {
    id: u32,
    client_id: u16,
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

#[derive(Debug)]
struct TransactionBodyWithAmount {
    id: u32,
    client_id: u16,
    amount: f64,
}

#[derive(Debug, Deserialize)]
struct TransactionCsvElement {
    pub action: String,
    pub client_id: u16,
    pub tx_id: u32,
    pub amount: Option<f64>
}

#[derive(Debug)]
enum Transaction {
    Deposit(TransactionBodyWithAmount),
    Withdraw(TransactionBodyWithAmount),
    Dispute(TransactionBody),
    Resolve(TransactionBody),
    Chargeback(TransactionBody),
}

impl From<TransactionCsvElement> for Transaction {
    fn from(tx: TransactionCsvElement) -> Transaction {
        match tx.action.as_str() {
            "deposit"  => Transaction::Deposit(TransactionBodyWithAmount { id: tx.tx_id, client_id: tx.client_id, amount: tx.amount.unwrap() }),
            "withdrawal" => Transaction::Withdraw(TransactionBodyWithAmount { id: tx.tx_id, client_id: tx.client_id, amount: tx.amount.unwrap() }),
            "dispute" => Transaction::Dispute(TransactionBody{ id: tx.tx_id, client_id: tx.client_id }),
            "resolve" => Transaction::Resolve(TransactionBody{ id: tx.tx_id, client_id: tx.client_id }),
            "chargeback" => Transaction::Chargeback(TransactionBody{ id: tx.tx_id, client_id: tx.client_id }) ,
            _ => panic!("can't parse transaction"),
        }
    }
}

#[derive(Default)]
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

    pub fn dispute(&mut self, amount: f64) {
        self.available -= amount;
        self.held += amount;
    }
}

struct Ledger {
    pub state: HashMap<u16, Account>,
    pub history: HashMap<u32, TransactionBodyWithAmount>,
}

impl Ledger {
    pub fn process_txs(&mut self, txs: Vec<Transaction>) {
        for tx in txs {
            match tx {
                Transaction::Deposit(e) => self.process_increment(e),
                Transaction::Withdraw(_) => todo!(),
                Transaction::Dispute(_) => todo!(),
                Transaction::Resolve(_) => todo!(),
                Transaction::Chargeback(_) => todo!(),
            }
        }
    }

    fn process_increment<G: ClientAndId + Amount>(&mut self, tx: G) {
        if !self.state.contains_key(&tx.client_id()) {
            self.state.insert(tx.client_id(), Account::default());
        }

        let account = self.state.get_mut(&tx.client_id()).unwrap();

        account.deposit(tx.amount());

        //self.history.insert(tx.id(), tx);
    }

    fn process_dispute(&mut self, tx: TransactionBody) {
        let account = self.state.get_mut(&tx.client_id()).unwrap();
        let disputed_tx = self.history.get(&tx.id()).unwrap();

        account.dispute(disputed_tx.amount());
    }
}

fn parse_csv() -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path("tests/test.csv")?;
    let mut all_transactions: Vec<Transaction> = Vec::new();
    for result in rdr.deserialize() {
        let record: TransactionCsvElement = result?;
        //all_transactions.push(record);
        all_transactions.push(Transaction::from(record));
    }
    //Transaction::from(all_transactions);
    //println!("{:?}", all_transactions);
    println!("{:?}", all_transactions);
    Ok((all_transactions))

}

fn main() {
    parse_csv();
}

