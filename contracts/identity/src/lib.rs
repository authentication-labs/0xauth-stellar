#![no_std]
use soroban_sdk::{
    contract, contractimpl, log, symbol_short, vec,
    xdr::{FromXdr, ToXdr},
    Address, Bytes, BytesN, Env, FromVal, Symbol, Vec,
};

mod state;

use state::{Claim, Key, KeyPurpose, KeyType};

#[contract]
pub struct IdentityContract;

#[contractimpl]
impl IdentityContract {
    pub fn get_initialized(env: Env) -> bool {
        env.storage()
            .instance()
            .get::<Symbol, bool>(&Symbol::new(&env, "initialized"))
            .unwrap_or(false)
    }

    pub fn initialize(env: Env, initial_management_key: Address) {
        initial_management_key.require_auth();

        let initialized = env
            .storage()
            .instance()
            .get::<Symbol, bool>(&Symbol::new(&env, "initialized"))
            .unwrap_or(false);

        if initialized {
            panic!("Contract already initialized");
        }
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "initialized"), &true);

        let key_hash = hash_key(&env, &initial_management_key);
        let key = Key {
            purposes: vec![&env, KeyPurpose::Management],
            key_type: KeyType::ECDSA,
            key: key_hash.clone(),
        };

        env.storage().persistent().set(&key_hash, &key);
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, "management_key"), &key_hash);

        let keys = vec![&env, key];
        env.storage()
            .persistent()
            .set(&symbol_short!("keys"), &keys);

        log!(
            &env,
            "Identity contract initialized with management key: {:?}",
            initial_management_key
        );
    }

    pub fn get_key(env: Env, key: Address) -> Key {
        let key_hash = hash_key(&env, &key);
        let keys = env
            .storage()
            .persistent()
            .get::<Symbol, Vec<Key>>(&symbol_short!("keys"))
            .unwrap_or(Vec::new(&env));

        keys.iter()
            .find(|k| k.key == key_hash)
            .expect("Key not found")
            .clone()
    }

    pub fn get_keys(env: Env) -> Vec<Key> {
        env.storage()
            .persistent()
            .get::<Symbol, Vec<Key>>(&symbol_short!("keys"))
            .unwrap_or(Vec::new(&env))
    }

    pub fn add_key(env: Env, manager: Address, key: Address, purpose: u32, key_type: u32) {
        // Only the manager can add keys
        identity_require_auth(&env, &manager, KeyPurpose::Management);

        // Make a Kecak256 hash of the key
        let key_hash = hash_key(&env, &key);
        let key_purpose = KeyPurpose::try_from(purpose).expect("Invalid key purpose");

        // Retrieve and mutate the list of keys
        let mut keys: Vec<Key> = env
            .storage()
            .persistent()
            .get::<Symbol, Vec<Key>>(&symbol_short!("keys"))
            .unwrap_or(Vec::new(&env));

        let mut key_found = false;

        for i in 0..keys.len() {
            let mut k = keys.get(i).expect("Index out of bounds");
            if k.key == key_hash {
                if k.purposes.contains(&key_purpose) {
                    panic!("Conflict: Key already exists with the same purpose");
                } else {
                    k.purposes.push_back(key_purpose);
                    key_found = true;
                    break;
                }
            }
        }

        if !key_found {
            let key = Key {
                purposes: vec![&env, key_purpose],
                key_type: KeyType::try_from(key_type).expect("Invalid key type"),
                key: key_hash.clone(),
            };
            keys.push_back(key);
        }
        env.storage()
            .persistent()
            .set(&symbol_short!("keys"), &keys);

        // TODO: Emit Key Add Event
    }

    pub fn remove_key(env: Env, manager: Address, key: Address, purpose: u32) {
        // Only the manager can remove keys
        identity_require_auth(&env, &manager, KeyPurpose::Management);

        // Make a Kecak256 hash of the key
        let key_hash = hash_key(&env, &key);
        let key_purpose = KeyPurpose::try_from(purpose).expect("Invalid key purpose");

        // Retrieve and mutate the list of keys
        let mut keys: Vec<Key> = env
            .storage()
            .persistent()
            .get::<Symbol, Vec<Key>>(&symbol_short!("keys"))
            .unwrap_or(Vec::new(&env));

        if !keys.iter().any(|k| k.key == key_hash) {
            panic!("Key not found");
        }

        for i in 0..keys.len() {
            let mut k = keys.get(i).expect("Index out of bounds");
            if k.key == key_hash {
                if let Some(pos) = k.purposes.iter().position(|p| p == key_purpose) {
                    k.purposes.remove(pos as u32);
                } else {
                    panic!("Key does not have the specified purpose");
                }

                if k.purposes.is_empty() {
                    keys.remove(i as u32);
                }
            }
        }

        

        log!(&env, "Key removed: {:?}", keys);


        env.storage()
            .persistent()
            .set(&symbol_short!("keys"), &keys);

        // TODO: Emit Key Removed Event
    }

    pub fn add_claim(
        env: Env,
        sender: Address,
        topic: u32,
        scheme: u32,
        issuer: Address,
        signature: Bytes,
        data: Bytes,
        uri: Bytes,
    ) {
        identity_require_auth(&env, &sender, KeyPurpose::Claim);
        // TODO: Make Contract Call isClaimValid

        let claim_id = hash_claim(&env, &issuer, topic);
        let claim: Claim = Claim {
            topic,
            scheme,
            issuer,
            signature,
            data,
            uri,
        };
        env.storage().persistent().set(&claim_id, &claim);

        // TODO: Call emitClaimAdded
        log!(&env, "Claim added: {:?}", claim);
    }

    pub fn remove_claim(env: Env, sender: Address, claim_id: BytesN<32>) {
        identity_require_auth(&env, &sender, KeyPurpose::Claim);

        let claim = env
            .storage()
            .persistent()
            .get::<BytesN<32>, Claim>(&claim_id)
            .expect("Claim not found");

        env.storage().persistent().remove(&claim_id);

        // TODO: Call emitClaimRemoved
        log!(&env, "Claim removed: {:?}", claim);
    }
}

fn hash_key(env: &Env, key: &Address) -> BytesN<32> {
    let address_bytes = Bytes::from_val(env, &key.to_xdr(&env));
    env.crypto().keccak256(&address_bytes)
}

fn hash_claim(env: &Env, issuer: &Address, topic: u32) -> BytesN<32> {
    let address_bytes = Bytes::from_val(env, &issuer.to_val());
    let topic_bytes = match Bytes::from_xdr(env, &topic.to_xdr(env)) {
        Ok(b) => b,
        Err(_) => panic!("Invalid topic"),
    };

    let mut concatenated_bytes = Bytes::new(env);
    concatenated_bytes.append(&address_bytes);
    concatenated_bytes.append(&topic_bytes);

    env.crypto().keccak256(&concatenated_bytes)
}

fn key_has_purpose(env: &Env, key_hash: &BytesN<32>, purpose: KeyPurpose) -> bool {
    if let Some(keys) = env
        .storage()
        .persistent()
        .get::<Symbol, Vec<Key>>(&symbol_short!("keys"))
    {
        keys.iter()
            .any(|k| k.key == *key_hash && k.purposes.contains(&purpose))
    } else {
        false
    }
}
fn identity_require_auth(env: &Env, sender: &Address, key_type: KeyPurpose) {
    let key_hash = hash_key(env, sender);

    if !key_has_purpose(env, &key_hash, key_type) {
        panic!("Permissions: Sender does not have the required key type");
    }

    sender.require_auth();
}

mod test;
