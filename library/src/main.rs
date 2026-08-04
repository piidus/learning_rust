use library::bank::{create_account, show_balance, transaction, TransactionStatus};
use std::ffi::CString;

fn main() {
    // Create accounts
    let owner1 = CString::new("SUDIIP").unwrap();
    let owner2 = CString::new("PIIDUS").unwrap();

    create_account(owner1.as_ptr(), 1, 100.00);
    create_account(owner2.as_ptr(), 2, 50.00);

    println!("--- Initial Balances ---");
    show_balance(1);
    show_balance(2);

    println!("\n--- Performing Transactions ---");
    handle_transaction(1, -25.50); // Withdraw 25.50 from account 1
    handle_transaction(2, 200.00); // Deposit 200.00 into account 2
    let status = transaction(3, 100.00); // Try to transact on a non-existent account
    println!(
        "Attempted transaction on non-existent account 3. Status: {:?}",
        status
    );

    println!("\n--- Final Balances ---");
    show_balance(1);
    show_balance(2);
}

fn handle_transaction(account_id: i32, amount: f64) {
    match transaction(account_id, amount) {
        TransactionStatus::Success => { /* Success message is already printed by the lib */ }
        TransactionStatus::AccountNotFound => println!("Error: Account {} not found.", account_id),
        TransactionStatus::MutexPoisoned => println!("Error: The bank library is in an inconsistent state."),
    }
}
