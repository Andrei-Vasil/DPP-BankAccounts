#[derive(Debug)]
pub struct Account {
    pub balance: f32,
}

impl Default for Account {
    fn default() -> Account {
        Account {
            balance: 50.0,
        }
    }
}

#[derive(Debug)]
pub struct OperationRecord {
    pub serial_number: i32,
    pub from: usize,
    pub to: usize,
    pub amount: f32
}