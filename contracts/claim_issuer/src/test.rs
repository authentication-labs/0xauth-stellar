#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::identity;

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ClaimIssuerContract);
    let client = ClaimIssuerContractClient::new(&env, &contract_id);

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

    let contract_id = env.register_contract(None, ClaimIssuerContract);
    let client = ClaimIssuerContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    let new_key = Address::generate(&env);
    client.add_key(&management_key, &new_key, &3, &1);

    // Verify that the key has been added
    let keys = client.get_keys();

    assert!(keys.iter().any(|k| k.key == hash_key(&env, &new_key)), "Key should be added");
}

#[test]
fn test_remove_key() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ClaimIssuerContract);
    let client = ClaimIssuerContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    let new_key = Address::generate(&env);
    client.add_key(&management_key, &new_key, &3, &1);

    client.remove_key(&management_key, &new_key, &3);

    // Verify that the key has been removed
    let keys = client.get_keys();

    assert!(!keys.iter().any(|k| k.key == hash_key(&env, &new_key)), "Key should be removed");
}

#[test]
fn test_add_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ClaimIssuerContract);
    let client = ClaimIssuerContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    let claim_key = Address::generate(&env);
    client.add_key(&management_key, &claim_key, &3, &1);

    let topic = U256::from_u32(&env, 6);
    let scheme = U256::from_u32(&env, 6);
    let issuer = Address::generate(&env);
    let signature = Bytes::from_val(&env, &"signature".to_xdr(&env));
    let data = Bytes::from_val(&env, &"data".to_xdr(&env));
    let uri = Bytes::from_val(&env, &"uri".to_xdr(&env));

    let claim_id = client.add_claim(&claim_key, &topic, &scheme, &issuer, &signature, &data, &uri);

    // Verify that the claim has been added
    let claim = match client.get_claim(&claim_id) {
        Some(claim) => claim,
        None => panic!("Claim not found"),
    };

    assert_eq!(claim.topic, topic, "Claim topic should match");
    assert_eq!(claim.scheme, scheme, "Claim scheme should match");
    assert_eq!(claim.issuer, issuer, "Claim issuer should match");
    assert_eq!(claim.signature, signature, "Claim signature should match");
    assert_eq!(claim.data, data, "Claim data should match");
    assert_eq!(claim.uri, uri, "Claim URI should match");
}

#[test]
fn test_remove_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ClaimIssuerContract);
    let client = ClaimIssuerContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    let claim_key = Address::generate(&env);
    client.add_key(&management_key, &claim_key, &3, &1);

    let topic = U256::from_u32(&env, 6);
    let scheme = U256::from_u32(&env, 6);
    let issuer = Address::generate(&env);
    let signature = Bytes::from_val(&env, &"signature".to_xdr(&env));
    let data = Bytes::from_val(&env, &"data".to_xdr(&env));
    let uri = Bytes::from_val(&env, &"uri".to_xdr(&env));

    client.add_claim(&claim_key, &topic, &scheme, &issuer, &signature, &data, &uri);

    let claim_id = hash_claim(&env, &issuer, &topic);

    client.remove_claim(&claim_key, &claim_id);

    // Verify that the claim has been removed
    let claim = client.get_claim(&claim_id);
    assert!(claim.is_none(), "Claim should be removed");
}


#[test]
fn test_revoke_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let identity_contract_id = env.register_contract_wasm(None, identity::WASM);

    let contract_id = env.register_contract(None, ClaimIssuerContract);
    let client = ClaimIssuerContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    let claim_key = Address::generate(&env);
    client.add_key(&management_key, &claim_key, &3, &1);

    let topic = U256::from_u32(&env, 6);
    let scheme = U256::from_u32(&env, 6);
    let issuer = Address::generate(&env);
    let signature = Bytes::from_val(&env, &"signature".to_xdr(&env));
    let data = Bytes::from_val(&env, &"data".to_xdr(&env));
    let uri = Bytes::from_val(&env, &"uri".to_xdr(&env));

    client.add_claim(&claim_key, &topic, &scheme, &issuer, &signature, &data, &uri);

    let claim_id = hash_claim(&env, &issuer, &topic);

    client.revoke_claim(&claim_key, &identity_contract_id, &claim_id);

    // Verify that the claim has been revoked
    let claim = client.is_claim_revoked(&signature);
    assert!(claim, "Claim should be revoked");
}

