#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};


#[test]
fn test_linking() {
    let env = Env::default();
    env.mock_all_auths();

    let factory_contract_id = env.register_contract(None, FactoryContract);
    let factory_client = FactoryContractClient::new(&env, &factory_contract_id);

    let owner = Address::generate(&env);
    let wallet = Address::generate(&env);
    let identity = Address::generate(&env);

    // Initialize the factory contract
    factory_client.initialize(&owner);

    // Link the wallet to the identity
    factory_client.link_wallet(&wallet, &identity);

    // Get the identity
    let linked_identity = factory_client.get_identity(&wallet);

    assert!(linked_identity == identity, "Identity should be linked to wallet");
}