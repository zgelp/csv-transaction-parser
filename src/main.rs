use serde::{Deserialize};
use std::error::Error;
use std::collections::HashMap;


trait ClientAndId {
    fn id(&self) -> u32;
    fn client_id(&self) -> u16;
}

trait Amount {
    fn amount(&self) -> f64;
}

#[derive(Debug, Deserialize)]
struct TransactionCsvElement {
    pub action: String,
    pub client_id: u16,
    pub tx_id: u32,
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
        let account = self.state.get_mut(&tx.client_id()).unwrap();
        if account.available > tx.amount() {
            account.withdrawal(tx.amount());
        }
    }

    fn process_dispute(&mut self, tx: TransactionBody) {
        let account = self.state.get_mut(&tx.client_id()).unwrap();
        let disputed_tx = self.history.get(&tx.id()).unwrap();
        account.dispute(disputed_tx.amount());
    }

    fn process_resolve(&mut self, tx: TransactionBody) {
        let account = self.state.get_mut(&tx.client_id()).unwrap();
        let resolved_tx = self.history.get(&tx.id()).unwrap();
        account.resolve_dispute(resolved_tx.amount());
    }

    fn process_chargeback(&mut self, tx: TransactionBody) {
        let account = self.state.get_mut(&tx.client_id()).unwrap();
        let chargeback_tx = self.history.get(&tx.id()).unwrap();
        if chargeback_tx.amount() <= account.held {
            account.chargeback(chargeback_tx.amount());
        }
    }
}

fn parse_csv() -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path("tests/test.csv")?;
    let mut all_transactions: Vec<Transaction> = Vec::new();
    for result in rdr.deserialize() {
        let record: TransactionCsvElement = result?;
        all_transactions.push(Transaction::from(record));
    }
    Ok(all_transactions)
}

fn main() {
    let transactions: Vec<Transaction> = parse_csv().unwrap();
    let mut ledger = Ledger::default();
    ledger.process_txs(transactions);
    println!("{:?}", ledger);
}