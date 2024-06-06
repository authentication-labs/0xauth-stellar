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

    // Verify that the key has been added
    let keys = client.get_keys();

    assert!(keys.iter().any(|k| k.key == hash_key(&env, &new_key)), "Key should be added");
}

#[test]
fn test_add_key_with_different_purpose() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, IdentityContract);
    let client = IdentityContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    client.add_key(&management_key, &management_key, &3, &1);

    // Verify that the key has been added
    let keys = client.get_keys();

    assert!(keys.iter().any(|k| k.key == hash_key(&env, &management_key)), "Key should be added");

    // Verify that the key has the correct purpose
    let key = keys.iter().find(|k| k.key == hash_key(&env, &management_key)).unwrap();

    // Check Purposes array
    assert_eq!(key.purposes.len(), 2, "Key should have two purpose");

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

    assert!(!keys.iter().any(|k| k.key == hash_key(&env, &new_key)), "Key should be removed");
}

#[test]
fn test_add_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, IdentityContract);
    let client = IdentityContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    let claim_key = Address::generate(&env);
    client.add_key(&management_key, &claim_key, &3, &1);

    let topic = U256::from_u32(&env, 6);
    let scheme = U256::from_u32(&env, 6);
    let issuer = contract_id;
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

    let contract_id = env.register_contract(None, IdentityContract);
    let client = IdentityContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    let claim_key = Address::generate(&env);
    client.add_key(&management_key, &claim_key, &3, &1);

    let topic = U256::from_u32(&env, 6);
    let scheme = U256::from_u32(&env, 6);
    let issuer = contract_id;
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
fn test_is_claim_valid() {
    // I made this Secret Key  for testing, do not use it anywhere else
    let sk = SigningKey::from_bytes(
        &hex::decode("b51a482a459d1b2f8f1ff5b7159cdbf0ab23ee46422ed0724f2822cd550ecf71")
            .unwrap()
            .try_into()
            .unwrap(),
    );

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, IdentityContract);
    let client = IdentityContractClient::new(&env, &contract_id);

    let management_key = Address::generate(&env);
    client.initialize(&management_key);

    let topic = U256::from_u32(&env, 6);
    let issuer = contract_id;
    let data = Bytes::from_val(&env, &"data".to_xdr(&env));

    let mut concatenated_bytes = Bytes::new(&env);
    concatenated_bytes.append(&issuer.clone().to_xdr(&env));
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

    let pk = sk.verifying_key();
    let pk_bytes: [u8; 32] = pk.to_bytes();

    if let Some(stellar_pub_key) = encode_stellar_pub_key(&pk_bytes) {
        std::println!("Stellar Public Key: {}", stellar_pub_key);
        let issuer_wallet = Address::from_string(&soroban_sdk::String::from_str(
            &env,
            stellar_pub_key.as_str(),
        ));

        client.add_key(&management_key, &issuer_wallet, &3, &1);

        let valid = client.is_claim_valid(&issuer_wallet, &issuer, &topic, &signature, &data);

        assert_eq!(valid, true, "Claim should be valid");
    } else {
        std::println!("Failed to encode the Stellar public key.");
    }
}

fn decode_stellar_pub_key(pub_key: &str) -> Option<[u8; 32]> {
    let decoded = base32_decode(Alphabet::Rfc4648 { padding: false }, pub_key)?;
    if decoded.len() != 35 {
        return None;
    }

    // std::println!("Decoded: {:?}", decoded);

    let checksum = &decoded[33..];
    let payload = &decoded[..33];

    // std::println!("Payload: {:?}", payload);
    // std::println!("Checksum: {:?}", checksum);

    // Verify checksum
    let mut state = State::<XMODEM>::new();
    state.update(payload);
    let calculated_checksum = state.get();

    // std::println!("Calculated Checksum: {:?}", calculated_checksum.to_le_bytes());

    if calculated_checksum.to_le_bytes() != [checksum[0], checksum[1]] {
        return None;
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&payload[1..33]);
    Some(key)
}

fn encode_stellar_pub_key(pub_key_bytes: &[u8; 32]) -> Option<String> {
    let mut payload = std::vec![0x30];
    payload.extend_from_slice(pub_key_bytes);

    let mut state = State::<XMODEM>::new();
    state.update(&payload);
    let checksum = state.get().to_le_bytes();

    std::println!("Payload: {:?}", payload);
    std::println!("Checksum: {:?}", checksum);

    payload.extend_from_slice(&checksum);

    let encoded = base32_encode(Alphabet::Rfc4648 { padding: false }, &payload);

    Some(encoded)
}
