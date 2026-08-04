#[repr(C)]
#[derive(Debug)]
pub enum TransactionStatus {
    Success = 0,
    AccountNotFound = 1,
    MutexPoisoned = 2,
}