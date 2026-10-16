use crate::model::{Claim, Contract};

pub fn save_claims(claims: Vec<Claim>) {
    //
    dbg!(claims);
}

pub fn save_contracts(contracts: Vec<Contract>) {
    dbg!(contracts);
}

#[cfg(test)]
//
mod test {

    fn test_save_claims() {}
}
