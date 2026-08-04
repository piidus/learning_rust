use super::accounts::ACCOUNTS;
use super::status::TransactionStatus;

// use for transaction
#[unsafe(no_mangle)]
pub extern "C" fn transaction(accountid: i32, amount: f64) -> TransactionStatus {
    let Ok(mut accounts) = ACCOUNTS.lock() else {
        // The mutex was poisoned. Return an error instead of panicking.
        eprintln!("Critical error: Mutex is poisoned.");
        return TransactionStatus::MutexPoisoned;
    };

    if let Some(account) = accounts.get_mut(&accountid) {
        // In a real-world scenario, you'd check for overdrafts, etc.
        account.balance += amount;
        println!("Transaction successful for account {}. New balance: {}", accountid, account.balance);
        TransactionStatus::Success
    } else {
        println!("Transaction failed: Account {} not found.", accountid);
        TransactionStatus::AccountNotFound
    }
}