#![cfg(test)]
extern crate std;
use super::*;
use ed25519_dalek::ed25519::signature::{Keypair, SignerMut};
use soroban_sdk::xdr::ScVal;
use soroban_sdk::{testutils::Address as _, Address, Env};
use soroban_sdk::testutils::ed25519::Sign;
use ed25519_dalek::SigningKey;
use base32::{Alphabet, encode};

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

// #[test]
// fn test_is_claim_valid() {

//     let sk = SigningKey::from_bytes(
//         &hex::decode("d1a54e6424182f5c0a83cb5b71a835ff407466b792ddb029fe4c53dcb8f18845")
//             .unwrap()
//             .try_into()
//             .unwrap(),
//     );

//     let env = Env::default();
//     env.mock_all_auths();

//     let contract_id = env.register_contract(None, IdentityContract);
//     let client = IdentityContractClient::new(&env, &contract_id);

//     let topic = U256::from_u32(&env, 6);
//     let issuer = contract_id;
//     let data = Bytes::from_val(&env, &"data".to_xdr(&env));

//     let mut concatenated_bytes = Bytes::new(&env);
//     concatenated_bytes.append(&issuer.clone().to_xdr(&env));
//     concatenated_bytes.append(&topic.clone().to_xdr(&env));
//     concatenated_bytes.append(&data);

//     let hashed_bytes: ScVal = env.crypto().keccak256(&concatenated_bytes).to_array().try_into().unwrap();
//     let sigtest_val: [u8; 64] = sk.sign(hashed_bytes).unwrap();

//     let signature = Bytes::from_slice(&env, &sigtest_val);

//     let pk = sk.verifying_key();
//     let public_key_bytes = BytesN::from_array(&env, &pk.to_bytes());

//     let valid = client.is_claim_valid(&public_key_bytes,&issuer, &topic, &signature, &data);

//     assert_eq!(valid, true, "Claim should be valid");
// }
