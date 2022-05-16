
#[derive(Debug, Deserialize)]
struct Transaction {
    action: String,
    client_id: u16,
    tx_id: u32,
    amount: f32
}


fn main() {
    println!("Hello, world!");
}
