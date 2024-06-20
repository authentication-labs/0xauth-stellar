#![cfg(test)]
extern crate std;

use super::*;
use base32::{decode as base32_decode, encode as base32_encode, Alphabet};
use crc16::{State, XMODEM};
use ed25519_dalek::SigningKey;
use soroban_sdk::testutils::ed25519::Sign;
use soroban_sdk::xdr::ScVal;
use soroban_sdk::{testutils::Address as _, Address, Env};
use std::string::String;

use crate::claim_issuer;
use crate::factory;
use crate::identity;

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let gated_contract_id = env.register_contract(None, GatedContract);
    let gated_client = GatedContractClient::new(&env, &gated_contract_id);

    let factory_contract_id = env.register_contract_wasm(None, factory::WASM);
    let claim_issuer_contract_id = env.register_contract_wasm(None, claim_issuer::WASM);

    let owner = Address::generate(&env);
    gated_client.initialize(&factory_contract_id, &claim_issuer_contract_id, &owner);

    assert_eq!(
        gated_client.get_initialized(),
        true,
        "Contract should be initialized"
    );
}
#[test]
fn test_validate_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let sk = SigningKey::from_bytes(
        &hex::decode("b51a482a459d1b2f8f1ff5b7159cdbf0ab23ee46422ed0724f2822cd550ecf71")
            .unwrap()
            .try_into()
            .unwrap(),
    );

    let issuer_wallet = Address::from_string(&soroban_sdk::String::from_str(
        &env,
        "GARPZXSZVTI7WG3ADZQB5QGH67WS6XW6ISHIO2N4AOAC32PLNYBH7QCY",
    ));


    let gated_contract_id = env.register_contract(None, GatedContract);
    let gated_client = GatedContractClient::new(&env, &gated_contract_id);

    let factory_contract_id = env.register_contract_wasm(None, factory::WASM);
    let factory_client = factory::Client::new(&env, &factory_contract_id);
    let claim_issuer_contract_id = env.register_contract_wasm(None, claim_issuer::WASM);
    let claim_issuer_client = claim_issuer::Client::new(&env, &claim_issuer_contract_id);
    let identity_contract_id = env.register_contract_wasm(None, identity::WASM);
    let identity_client = identity::Client::new(&env, &identity_contract_id);

    
    let management_key = Address::generate(&env);
    factory_client.initialize(&management_key);
    claim_issuer_client.initialize(&management_key);
    identity_client.initialize(&management_key);
    gated_client.initialize(&factory_contract_id, &claim_issuer_contract_id, &management_key);

    // Add Claim Key 
    let claim_key = Address::generate(&env);
    identity_client.add_key(&management_key, &claim_key, &3, &1);
    claim_issuer_client.add_key(&management_key, &issuer_wallet, &3, &1);

    // Link Identity to User Wallet
    let user_wallet = Address::generate(&env);
    factory_client.link_wallet(&user_wallet, &identity_contract_id);

    // Create Claim
    let topic = U256::from_u32(&env, 6);
    let scheme = U256::from_u32(&env, 6);
    let issuer = claim_issuer_contract_id;
    let data = "data".to_xdr(&env);
    let uri = Bytes::from_val(&env, &"uri".to_xdr(&env));

    let mut concatenated_bytes = Bytes::new(&env);
    concatenated_bytes.append(&identity_contract_id.clone().to_xdr(&env));
    concatenated_bytes.append(&topic.clone().to_xdr(&env));
    concatenated_bytes.append(&data);

    let hashed_bytes: ScVal = env
        .crypto()
        .keccak256(&concatenated_bytes)
        .to_array()
        .try_into()
        .unwrap();

    let sigtest_val: [u8; 64] = sk.sign(hashed_bytes).unwrap();
    let signature = Bytes::from_slice(&env, &sigtest_val);

    let claim_id = identity_client.add_claim(
        &claim_key,
        &topic,
        &scheme,
        &issuer_wallet,
        &issuer,
        &signature,
        &data,
        &uri,
    );

    // Verify that the claim has been added
    match identity_client.get_claim(&claim_id) {
        Some(claim) => claim,
        None => panic!("Claim not found"),
    };

    assert_eq!(
        gated_client.validate_claim(&user_wallet, &issuer_wallet, &topic),
        true,
        "Claim should be valid"
    );

    
}
