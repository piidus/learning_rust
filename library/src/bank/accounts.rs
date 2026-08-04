use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::ffi::{c_char, CStr};

pub(crate) static ACCOUNTS: Lazy<Mutex<HashMap<i32, Account>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

#[repr(C)]
#[derive(Debug)]
pub struct Account{
    pub owner:String,
    pub ac_no:i32,
    pub balance:f64
}

#[unsafe(no_mangle)] // This is unsafe because it's creating a C-style, unmangled symbol
pub extern "C" fn create_account(owner: *const c_char, ac_no: i32, balance: f64) {
    let owner_str = unsafe {
        assert!(!owner.is_null());
        CStr::from_ptr(owner).to_str().unwrap_or("").to_string()
    };

    if !owner_str.is_empty() {
        let mut accounts = ACCOUNTS.lock().unwrap();
        let account = Account { owner: owner_str, ac_no, balance };
        accounts.insert(ac_no, account);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn show_balance(accountid:i32){
    let accounts = ACCOUNTS.lock().unwrap();
    if let Some(account) = accounts.get(&accountid) {
        println!("Account {:?} balance is: {}", accountid, account.balance);
        dbg!(account);
    } else {
        println!("Account {} not found.", accountid);
    }
}