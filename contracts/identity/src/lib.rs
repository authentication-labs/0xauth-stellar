#![no_std]
use soroban_sdk::{
    contract, contractimpl, log, symbol_short, vec, xdr::ToXdr, Address, Bytes, BytesN, Env,
    FromVal, Symbol, Vec, U256,
};

mod state;
use state::{Claim, Error, Key, KeyPurpose, KeyType};

mod claim_issuer {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/claim_issuer.wasm"
    );
}

#[contract]
pub struct IdentityContract;

#[contractimpl]
impl IdentityContract {
    pub fn get_initialized(env: Env) -> Result<bool, Error> {
        Ok(env
            .storage()
            .instance()
            .get::<Symbol, bool>(&Symbol::new(&env, "initialized"))
            .unwrap_or(false))
    }

    pub fn initialize(env: Env, initial_management_key: Address) -> Result<(), Error> {
        let initialized = env
            .storage()
            .instance()
            .get::<Symbol, bool>(&Symbol::new(&env, "initialized"))
            .unwrap_or(false);

        if initialized {
            return Err(Error::AlreadyInitialized);
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

        let keys = vec![&env, key];
        env.storage()
            .persistent()
            .set(&symbol_short!("keys"), &keys);

        log!(
            &env,
            "Identity contract initialized with management key: {:?}",
            initial_management_key
        );
        Ok(())
    }

    pub fn get_key(env: Env, key: Address) -> Result<Key, Error> {
        let key_hash = hash_key(&env, &key);
        let keys = env
            .storage()
            .persistent()
            .get::<Symbol, Vec<Key>>(&symbol_short!("keys"))
            .unwrap_or(Vec::new(&env));

        keys.iter()
            .find(|k| k.key == key_hash)
            .clone()
            .ok_or(Error::KeyNotFound)
    }

    pub fn get_keys(env: Env) -> Result<Vec<Key>, Error> {
        Ok(env
            .storage()
            .persistent()
            .get::<Symbol, Vec<Key>>(&symbol_short!("keys"))
            .unwrap_or(Vec::new(&env)))
    }

    pub fn add_key(
        env: Env,
        manager: Address,
        key: Address,
        purpose: u32,
        key_type: u32,
    ) -> Result<(), Error> {
        // Only the manager can add keys
        identity_require_auth(&env, &manager, KeyPurpose::Management)?;

        // Make a Kecak256 hash of the key
        let key_hash = hash_key(&env, &key);
        let key_purpose = KeyPurpose::try_from(purpose).map_err(|_| Error::InvalidKeyPurpose)?;

        // Retrieve and mutate the list of keys
        let mut keys: Vec<Key> = env
            .storage()
            .persistent()
            .get::<Symbol, Vec<Key>>(&symbol_short!("keys"))
            .unwrap_or(Vec::new(&env));

        let mut key_found = false;

        for i in 0..keys.len() {
            let mut k = keys.get(i).ok_or(Error::IndexOutOfBounds)?;
            if k.key == key_hash {
                if k.purposes.contains(&key_purpose) {
                    return Err(Error::KeyConflict);
                } else {
                    k.purposes.push_back(key_purpose);
                    keys.set(i, k);
                    key_found = true;
                    break;
                }
            }
        }

        if !key_found {
            let key = Key {
                purposes: vec![&env, key_purpose],
                key_type: KeyType::try_from(key_type).map_err(|_| Error::InvalidKeyType)?,
                key: key_hash.clone(),
            };
            keys.push_back(key);
        }
        env.storage()
            .persistent()
            .set(&symbol_short!("keys"), &keys);

        // TODO: Emit Key Add Event
        Ok(())
    }

    pub fn remove_key(env: Env, manager: Address, key: Address, purpose: u32) -> Result<(), Error> {
        // Only the manager can remove keys
        identity_require_auth(&env, &manager, KeyPurpose::Management)?;

        // Make a Kecak256 hash of the key
        let key_hash = hash_key(&env, &key);
        let key_purpose = KeyPurpose::try_from(purpose).map_err(|_| Error::InvalidKeyPurpose)?;

        // Retrieve and mutate the list of keys
        let mut keys: Vec<Key> = env
            .storage()
            .persistent()
            .get::<Symbol, Vec<Key>>(&symbol_short!("keys"))
            .unwrap_or(Vec::new(&env));

        if !keys.iter().any(|k| k.key == key_hash) {
            return Err(Error::KeyNotFound);
        }

        for i in 0..keys.len() {
            if let Some(mut k) = keys.get(i) {
                if k.key == key_hash {
                    if let Some(pos) = k.purposes.iter().position(|p| p == key_purpose) {
                        k.purposes.remove(pos as u32);

                        if k.purposes.is_empty() {
                            keys.remove(i as u32);
                        } else {
                            keys.set(i, k);
                        }
                    } else {
                        return Err(Error::KeyDoesNotHavePurpose);
                    }
                    break;
                }
            } else {
                return Err(Error::IndexOutOfBounds);
            }
        }

        env.storage()
            .persistent()
            .set(&symbol_short!("keys"), &keys);
        // TODO: Emit Key Removed Event
        Ok(())
    }

    pub fn get_claim(env: Env, claim_id: BytesN<32>) -> Result<Option<Claim>, Error> {
        Ok(env
            .storage()
            .persistent()
            .get::<BytesN<32>, Claim>(&claim_id))
    }

    pub fn get_claim_ids(env: Env) -> Result<Vec<BytesN<32>>, Error> {
        Ok(env
            .storage()
            .persistent()
            .get::<Symbol, Vec<BytesN<32>>>(&symbol_short!("claims"))
            .unwrap_or(Vec::new(&env)))
    }

    pub fn add_claim(
        env: Env,
        sender: Address,
        topic: U256,
        scheme: U256,
        issuer: Address,
        signature: Bytes,
        data: Bytes,
        uri: Bytes,
    ) -> Result<BytesN<32>, Error> {
        identity_require_auth(&env, &sender, KeyPurpose::Claim)?;

        let current_contact = env.current_contract_address();

        if current_contact != issuer {
            let client = claim_issuer::Client::new(&env, &issuer);
            if client.is_claim_valid(&issuer, &topic, &signature, &data) {
                return Err(Error::InvalidClaim);
            }
        }

        let claim_id = hash_claim(&env, &issuer, &topic);
        let claim: Claim = Claim {
            topic,
            scheme,
            issuer,
            signature,
            data,
            uri,
        };
        env.storage().persistent().set(&claim_id, &claim);

        let mut claims = env
            .storage()
            .persistent()
            .get::<Symbol, Vec<BytesN<32>>>(&symbol_short!("claims"))
            .unwrap_or(Vec::new(&env));

        claims.push_back(claim_id.clone());

        // TODO: Call emitClaimAdded
        log!(&env, "Claim added: {:?}", claim);

        Ok(claim_id)
    }

    pub fn remove_claim(env: Env, sender: Address, claim_id: BytesN<32>) -> Result<(), Error> {
        identity_require_auth(&env, &sender, KeyPurpose::Claim)?;

        let claim = env
            .storage()
            .persistent()
            .get::<BytesN<32>, Claim>(&claim_id)
            .ok_or(Error::ClaimNotFound)?;

        env.storage().persistent().remove(&claim_id);

        let mut claims = env
            .storage()
            .persistent()
            .get::<Symbol, Vec<BytesN<32>>>(&symbol_short!("claims"))
            .unwrap_or(Vec::new(&env));

        if let Some(pos) = claims.iter().position(|c| c == claim_id) {
            claims.remove(pos as u32);
        }

        env.storage()
            .persistent()
            .set(&symbol_short!("claims"), &claims);

        // TODO: Call emitClaimRemoved
        log!(&env, "Claim removed: {:?}", claim);
        Ok(())
    }

    pub fn is_claim_valid(
        env: &Env,
        issuer: Address,
        topic: U256,
        signature: Bytes,
        data: Bytes,
    ) -> Result<bool, Error> {
        let address_bytes = Bytes::from_val(env, &issuer.to_xdr(&env));
        let topic_bytes = Bytes::from_val(env, &topic.to_xdr(env));

        let mut concatenated_bytes = Bytes::new(env);
        concatenated_bytes.append(&address_bytes);
        concatenated_bytes.append(&topic_bytes);
        concatenated_bytes.append(&data);

        let data_hash = env.crypto().keccak256(&concatenated_bytes);

        let mut digest_bytes = Bytes::from_slice(&env, b"\x19Ethereum Signed Message:\n32");
        digest_bytes.append(&data_hash.to_xdr(env));
        let prefixed_hash = env.crypto().keccak256(&digest_bytes);

        let signature_slice: BytesN<64> = signature
            .slice(..64)
            .try_into()
            .map_err(|_| Error::InvalidSignature)?;

        let recovery_id = match signature.get(64) {
            Some(v) => (v + 27) as u32,
            None => return Err(Error::InvalidSignature),
        };

        let recovered: BytesN<65> =
            env.crypto()
                .secp256k1_recover(&prefixed_hash, &signature_slice, recovery_id);
        let recovered_addr = Address::from_string_bytes(&recovered.to_xdr(env));
        let hashed_addr = env.crypto().keccak256(&recovered_addr.to_xdr(env));

        Ok(key_has_purpose(env, &hashed_addr, KeyPurpose::Claim))
    }
}

fn hash_key(env: &Env, key: &Address) -> BytesN<32> {
    let address_bytes = Bytes::from_val(env, &key.to_xdr(&env));
    env.crypto().keccak256(&address_bytes)
}

fn hash_claim(env: &Env, issuer: &Address, topic: &U256) -> BytesN<32> {
    let address_bytes = Bytes::from_val(env, &issuer.to_xdr(&env));
    let topic_bytes = Bytes::from_val(env, &topic.to_xdr(env));

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
fn identity_require_auth(env: &Env, sender: &Address, key_type: KeyPurpose) -> Result<(), Error> {
    let key_hash = hash_key(env, sender);

    if !key_has_purpose(env, &key_hash, key_type) {
        return Err(Error::InsufficientPermissions);
    }

    sender.require_auth();
    Ok(())
}

mod test;
