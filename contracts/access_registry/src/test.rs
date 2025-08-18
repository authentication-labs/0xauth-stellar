#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Env, String};

#[test]
fn test() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AccessRegistryContract, ());
    let client = AccessRegistryContractClient::new(&env, &contract_id);

    let fund_id = String::from_str(&env, "abc123");
    let admin = Address::generate(&env);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);

    assert_eq!(client.get_initialized(), false);

    client.initialize(&fund_id, &admin);
    assert_eq!(client.get_initialized(), true);

    assert_eq!(client.get_fundid().unwrap(), fund_id.clone());

    assert_eq!(client.get_admin().unwrap(), admin.clone());
    assert_eq!(client.is_whitelisted(&user_a), false);
    assert_eq!(client.is_whitelisted(&user_b), false);

    client.add_address(&admin, &user_a);
    assert_eq!(client.is_whitelisted(&user_a), true);

    client.add_address(&admin, &user_b);
    assert_eq!(client.is_whitelisted(&user_a), true);

    client.remove_address(&admin, &user_a);
    assert_eq!(client.is_whitelisted(&user_a), false);
    assert_eq!(client.is_whitelisted(&user_b), true);

    client.add_address(&admin, &user_a);
    assert_eq!(client.is_whitelisted(&user_a), true);
}
