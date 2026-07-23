use library:: bank :: {Account,show_balance};
fn main() {
    let u1 = Account{
        owner:"Sudiip".to_uppercase(),
        ac_no: 1,
        balance:0.00
    };
    let u2 = Account{
        owner:"Piidus".to_uppercase(),
        ac_no: 2,
        balance:0.00
    };
    show_balance(u1.ac_no);
    show_balance(u2.ac_no);
}
