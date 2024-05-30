#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, IdentityContract);
    let client = IdentityContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    assert_eq!(
        client.get_initialized(),
        true,
        "Contract should be initialized"
    );
}

#[test]
fn test_add_key() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, IdentityContract);
    let client = IdentityContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    let new_key = Address::generate(&env);
    client.add_key(&management_key, &new_key, &3, &1);

    // // Verify that the key has been added
    let keys = client.get_keys();

    assert!(keys.iter().any(|k| k.key == hash_key(&env, &new_key)), "Key should be added");
}

#[test]
fn test_remove_key() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, IdentityContract);
    let client = IdentityContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    let new_key = Address::generate(&env);
    client.add_key(&management_key, &new_key, &3, &1);

    client.remove_key(&management_key, &new_key, &3);

    // Verify that the key has been removed
    let keys = client.get_keys();

    let res = !keys.iter().any(|k| k.key == hash_key(&env, &new_key));

    assert!(res, "Key should be removed");
}

// #[test]
// fn test_add_claim() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let contract_id = env.register_contract(None, IdentityContract);
//     let client = IdentityContractClient::new(&env, &contract_id);

//     let management_key = Address::generate(&env);
//     client.initialize(&management_key);

//     let claim_key = Address::generate(&env);
//     client.add_key(&management_key, &claim_key, &3, &1);

//     let topic = 1;
//     let scheme = 1;
//     let issuer = Address::generate(&env);
//     let signature = Bytes::from_val(&env, &"signature".to_xdr(&env));
//     let data = Bytes::from_val(&env, &"data".to_xdr(&env));
//     let uri = Bytes::from_val(&env, &"uri".to_xdr(&env));

//     client.add_claim(&claim_key, &topic, &scheme, &issuer, &signature, &data, &uri);

//     // Verify that the claim has been added
//     let claim_id = hash_claim(&env, &issuer, topic);
//     let claim = env
//         .storage()
//         .persistent()
//         .get::<BytesN<32>, Claim>(&claim_id)
//         .unwrap_or_else(|| panic!("Claim not found"));

//     assert_eq!(claim.topic, topic, "Claim topic should match");
//     assert_eq!(claim.scheme, scheme, "Claim scheme should match");
//     assert_eq!(claim.issuer, issuer, "Claim issuer should match");
//     assert_eq!(claim.signature, signature, "Claim signature should match");
//     assert_eq!(claim.data, data, "Claim data should match");
//     assert_eq!(claim.uri, uri, "Claim URI should match");
// }

// #[test]
// fn test_remove_claim() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let contract_id = env.register_contract(None, IdentityContract);
//     let client = IdentityContractClient::new(&env, &contract_id);

//     let management_key = Address::generate(&env);
//     client.initialize(&management_key);

//     let claim_key = Address::generate(&env);
//     client.add_key(&management_key, &claim_key, &3, &1);

//     let topic = 1;
//     let scheme = 1;
//     let issuer = Address::generate(&env);
//     let signature = Bytes::from_val(&env, &"signature".to_xdr(&env));
//     let data = Bytes::from_val(&env, &"data".to_xdr(&env));
//     let uri = Bytes::from_val(&env, &"uri".to_xdr(&env));

//     client.add_claim(&claim_key, &topic, &scheme, &issuer, &signature, &data, &uri);

//     let claim_id = hash_claim(&env, &issuer, topic);

//     client.remove_claim(&claim_key, &claim_id);

//     // Verify that the claim has been removed
//     let claim = env.storage().persistent().get::<BytesN<32>, Claim>(&claim_id);
//     assert!(claim.is_none(), "Claim should be removed");
// }
