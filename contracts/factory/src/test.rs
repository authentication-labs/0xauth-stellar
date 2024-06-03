#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, vec, Env};

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, FactoryContract);
    let client = FactoryContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    client.initialize(&owner);
    assert_eq!(
        client.get_initialized(),
        true,
    );
}
