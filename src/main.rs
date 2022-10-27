// http://www.cs.ubbcluj.ro/~rlupsa/edu/pdp/lab-1-noncooperative-mt.html
// ex2 - bank accounts

mod json;
mod account;

use json::Operation;
use account::{Account, OperationRecord};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

fn increment_serial_number(serial_number_count: &mut Arc<Mutex<i32>>) -> i32 {
    let mut guard = serial_number_count.lock().unwrap();
    let serial_number: i32 = *guard;
    *guard += 1;
    return serial_number;
}

fn atomic_transaction(account: &Mutex<Account>, amount: f32) {
    let mut guard = account.lock().unwrap();
    guard.balance += amount;
}

fn transaction(from: &Mutex<Account>, to: &Mutex<Account>, amount: f32, checking: Arc<RwLock<i32>>) {    
    let rl_guard = checking.read().unwrap();
    assert_eq!(*rl_guard, 0);

    atomic_transaction(from, -amount);
    atomic_transaction(to, amount);
}

fn append_operation_record(records: Arc<Mutex<Vec<OperationRecord>>>, record_to_append: OperationRecord, checking: Arc<RwLock<i32>>) {
    let rl_guard = checking.read().unwrap();
    assert_eq!(*rl_guard, 0);

    let mut guard = records.lock().unwrap();
    guard.push(record_to_append);
}

fn check_all(records: Arc<Mutex<Vec<OperationRecord>>>, accounts: Arc<Vec<Mutex<Account>>>, checking: Arc<RwLock<i32>>) {
    let wl_guard = checking.write().unwrap();
    assert_eq!(*wl_guard, 0);
    
    let mut accounts_check: Vec<Account> = vec![];
    for account in Arc::clone(&accounts).iter() {
        accounts_check.push(Account {
            balance: account.lock().unwrap().balance
        });
    }
    for record in records.lock().unwrap().iter() {
        accounts_check[record.from].balance += record.amount;
        accounts_check[record.to].balance -= record.amount;
    }
    for account in accounts_check {
        assert_eq!(account.balance, 50.0);
    }
}

fn main() {
    const QUANTIFY_CHECK_VALUE: i32 = 2;

    let mut handles = vec![];

    let mut serial_number_count: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
    let records: Arc<Mutex<Vec<OperationRecord>>> = Arc::new(Mutex::new(vec![]));
    let accounts = Arc::new(vec![
        Mutex::new(Account {..Default::default()}), 
        Mutex::new(Account {..Default::default()}),
        Mutex::new(Account {..Default::default()}),
        Mutex::new(Account {..Default::default()})
    ]);
    let checking = Arc::new(RwLock::new(0));

    let mut check_idx = 0;

    let text: String = std::fs::read_to_string("input.json").unwrap();
    let operations: Vec<Operation> = serde_json::from_str(&text).unwrap();
    for operation in operations {
        if operation.from == operation.to {
            continue;
        }

        let checking_clone: Arc<RwLock<i32>> = Arc::clone(&checking);
        let accounts_clone: Arc<Vec<Mutex<Account>>> = Arc::clone(&accounts);
        let records_clone: Arc<Mutex<Vec<OperationRecord>>> = Arc::clone(&records);
        let mut serial_number_count_clone = Arc::clone(&mut serial_number_count);
        let transaction_handle = thread::spawn(move || {
            let operation_record = OperationRecord {
                serial_number: increment_serial_number(&mut serial_number_count_clone),
                from: operation.from,
                to: operation.to,
                amount: operation.amount
            };
            append_operation_record(records_clone, operation_record, Arc::clone(&checking_clone));

            transaction(&accounts_clone[operation.from], &accounts_clone[operation.to], operation.amount, Arc::clone(&checking_clone));
        });
        handles.push(transaction_handle);
        
        if check_idx == 0 {
            let checking_clone: Arc<RwLock<i32>> = Arc::clone(&checking);
            let records_clone: Arc<Mutex<Vec<OperationRecord>>> = Arc::clone(&records);
            let accounts_clone: Arc<Vec<Mutex<Account>>> = Arc::clone(&accounts);
            let check_handle = thread::spawn(move || {
                check_all(records_clone, accounts_clone, checking_clone);
            });
            handles.push(check_handle);
        }
        check_idx += 1;
        check_idx %= QUANTIFY_CHECK_VALUE;
    }

    let checking_clone: Arc<RwLock<i32>> = Arc::clone(&checking);
    let records_clone: Arc<Mutex<Vec<OperationRecord>>> = Arc::clone(&records);
    let accounts_clone: Arc<Vec<Mutex<Account>>> = Arc::clone(&accounts);
    let check_handle = thread::spawn(move || {
        check_all(records_clone, accounts_clone, checking_clone);
    });
    handles.push(check_handle);

    for handle in handles {
        handle.join().unwrap();
    }

    
}
