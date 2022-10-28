#[derive(Debug)]
pub struct Account {
    pub balance: i32,
}

impl Default for Account {
    fn default() -> Account {
        Account {
            balance: 5000,
        }
    }
}

#[derive(Debug)]
pub struct OperationRecord {
    pub serial_number: i32,
    pub from: usize,
    pub to: usize,
    pub amount: i32
}