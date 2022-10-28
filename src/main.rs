// http://www.cs.ubbcluj.ro/~rlupsa/edu/pdp/lab-1-noncooperative-mt.html
// ex2 - bank accounts

mod json;
mod account;

use json::Operation;
use account::{Account, OperationRecord};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use rand::Rng;

fn increment_serial_number(serial_number_count: &mut Arc<Mutex<i32>>) -> i32 {
    let mut guard = serial_number_count.lock().unwrap();
    let serial_number: i32 = *guard;
    *guard += 1;
    return serial_number;
}

fn atomic_transaction(account: &Mutex<Account>, amount: i32) {
    let mut guard = account.lock().unwrap();
    guard.balance += amount;
}

fn transaction(from: &Mutex<Account>, to: &Mutex<Account>, amount: i32) {    
    atomic_transaction(from, -amount);
    atomic_transaction(to, amount);
}

fn append_operation_record(records: Arc<Mutex<Vec<OperationRecord>>>, record_to_append: OperationRecord) {
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

    let mut i: i32 = 0;
    for account in accounts_check {
        if account.balance != 5000 {
            println!("{:?} {:?}", account, i);
        }
        i += 1;
        assert_eq!(account.balance, 5000);
    }
}

fn generate_accounts() -> Vec<Mutex<Account>> {
    let mut rng = rand::thread_rng();
    let mut accounts: Vec<Mutex<Account>> = vec![];
    
    for _ in 0..rng.gen_range(2..10) {
        accounts.push(Mutex::new(Account {..Default::default()}));
    }
    return accounts;
}

fn generate_operations(max_idx: usize) -> Vec<Operation> {
    let mut rng = rand::thread_rng();
    let mut operations: Vec<Operation> = vec![];
    
    for _ in 0..rng.gen_range(1..10000) {
        let mut operation = Operation {
            from: rng.gen_range(0..max_idx),
            to: rng.gen_range(0..max_idx),
            amount: rng.gen_range(0..5)
        };
        while operation.from == operation.to {
            operation = Operation {
                from: rng.gen_range(0..max_idx),
                to: rng.gen_range(0..max_idx),
                amount: rng.gen_range(0..5)
            };  
        }
        operations.push(operation);
    }
    // println!("{:?}", operations);
    return operations;
}

fn main() {
    const QUANTIFY_CHECK_VALUE: i32 = 2;

    let mut handles = vec![];

    let mut serial_number_count: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
    let records: Arc<Mutex<Vec<OperationRecord>>> = Arc::new(Mutex::new(vec![]));
    let accounts = Arc::new(generate_accounts());
    let checking = Arc::new(RwLock::new(0));

    let mut check_idx = 0;

    // let text: String = std::fs::read_to_string("input.json").unwrap();
    // let operations: Vec<Operation> = serde_json::from_str(&text).unwrap();
    let operations: Vec<Operation> = generate_operations(accounts.len());
    for operation in operations {
        if operation.from == operation.to {
            continue;
        }

        let checking_clone: Arc<RwLock<i32>> = Arc::clone(&checking);
        let accounts_clone: Arc<Vec<Mutex<Account>>> = Arc::clone(&accounts);
        let records_clone: Arc<Mutex<Vec<OperationRecord>>> = Arc::clone(&records);
        let mut serial_number_count_clone = Arc::clone(&mut serial_number_count);
        let transaction_handle = thread::spawn(move || {
            let rl_guard = checking_clone.read().unwrap();
            assert_eq!(*rl_guard, 0);

            let operation_record = OperationRecord {
                serial_number: increment_serial_number(&mut serial_number_count_clone),
                from: operation.from,
                to: operation.to,
                amount: operation.amount
            };
            append_operation_record(records_clone, operation_record);

            transaction(&accounts_clone[operation.from], &accounts_clone[operation.to], operation.amount);
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
