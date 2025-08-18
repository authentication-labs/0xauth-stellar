#![no_std]

use soroban_sdk::{
    auth::CustomAccountInterface, contract, contractimpl, contracttype, crypto::Hash, log,
    symbol_short, vec, xdr::ToXdr, Address, Bytes, BytesN, Env, FromVal, Symbol, Vec, U256,
};

mod state;
use state::{Claim, Error, Key, KeyPurpose, KeyType};

mod claim_issuer {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/claim_issuer.wasm");
}

const STORAGE_KEY_PK: Symbol = symbol_short!("pk");

#[contract]
pub struct IdentityContract;

#[contractimpl]
impl IdentityContract {
    pub fn extend_ttl(env: Env) {
        let max_ttl = env.storage().max_ttl();

        env.storage().instance().extend_ttl(max_ttl, max_ttl);
    }

    pub fn get_initialized(env: Env) -> Result<bool, Error> {
        Ok(env
            .storage()
            .instance()
            .get::<Symbol, bool>(&Symbol::new(&env, "initialized"))
            .unwrap_or(false))
    }

    pub fn initialize(
        env: Env,
        initial_management_key: Address,
        passkey_pk: BytesN<65>,
    ) -> Result<(), Error> {
        let init_symbol = Symbol::new(&env, "initialized");

        let initialized = env
            .storage()
            .instance()
            .get::<Symbol, bool>(&init_symbol)
            .unwrap_or(false);

        if initialized {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&init_symbol, &true);

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

        env.storage().instance().set(&STORAGE_KEY_PK, &passkey_pk);

        log!(
            &env,
            "Identity contract initialized with management key: {:?}",
            initial_management_key
        );

        env.events().publish((init_symbol,), initial_management_key);
        Self::extend_ttl(env);
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

        env.events().publish(
            (symbol_short!("add_key"),),
            (manager, key, purpose, key_type),
        );
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

        env.events()
            .publish((Symbol::new(&env, "remove_key"),), (manager, key, purpose));
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
        issuer_wallet: Address,
        issuer: Address,
        signature: Bytes,
        data: Bytes,
        uri: Bytes,
    ) -> Result<BytesN<32>, Error> {
        identity_require_auth(&env, &sender, KeyPurpose::Claim)?;

        let current_contact = env.current_contract_address();

        if current_contact != issuer {
            let client = claim_issuer::Client::new(&env, &issuer);
            if !client.is_claim_valid(&issuer_wallet, &current_contact, &topic, &signature, &data) {
                return Err(Error::InvalidClaim);
            }
        }

        let claim_id = hash_claim(&env, &issuer, &topic);
        let claim: Claim = Claim {
            topic,
            scheme,
            issuer,
            issuer_wallet,
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

        env.storage()
            .persistent()
            .set(&symbol_short!("claims"), &claims);

        log!(&env, "Claim added: {:?}", claim);

        env.events().publish(
            (symbol_short!("add_claim"),),
            (
                sender,
                claim_id.clone(),
                claim.topic,
                claim.scheme,
                claim.issuer,
                claim.issuer_wallet,
                claim.signature,
                claim.data,
                claim.uri,
            ),
        );

        Ok(claim_id)
    }

    pub fn get_publickey(env: Env) -> Result<BytesN<65>, Error> {
        env.storage()
            .instance()
            .get::<Symbol, BytesN<65>>(&STORAGE_KEY_PK)
            .ok_or(Error::NotInitialized)
    }
    pub fn remove_claim(env: Env, sender: Address, claim_id: BytesN<32>) -> Result<(), Error> {
        // check if sender has `claim key` otherwise check if passkey hash authorized
        if identity_require_auth(&env, &sender, KeyPurpose::Claim).is_err() {
            env.current_contract_address().require_auth();
        }

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

        log!(&env, "Claim removed: {:?}", claim);

        env.events()
            .publish((Symbol::new(&env, "remove_claim"),), (sender, claim_id));
        Ok(())
    }

    pub fn is_claim_valid(
        env: &Env,
        issuer_wallet: Address,
        identity: Address,
        topic: U256,
        signature: Bytes,
        data: Bytes,
    ) -> Result<bool, Error> {
        let mut concatenated_bytes = Bytes::new(env);
        concatenated_bytes.append(&identity.clone().to_xdr(&env));
        concatenated_bytes.append(&topic.to_xdr(&env));
        concatenated_bytes.append(&data);

        // Make sure the Signature was also signed in the XDR format
        // Otherwise, the signature will be invalid
        let data_digest = env.crypto().keccak256(&concatenated_bytes).to_xdr(&env);

        let signature_slice: BytesN<64> = match signature.slice(..64).try_into() {
            Ok(slice) => slice,
            Err(_) => return Err(Error::InvalidSignature),
        };

        let issuer_xdr = issuer_wallet.clone().to_xdr(env);

        let issuer_bytes: BytesN<32> = match issuer_xdr.slice(12..44).try_into() {
            Ok(slice) => slice,
            Err(_) => return Err(Error::InvalidAddressBytes),
        };

        env.crypto()
            .ed25519_verify(&issuer_bytes, &data_digest, &signature_slice);

        let hashed_addr = hash_key(env, &issuer_wallet);

        Ok(key_has_purpose(env, &hashed_addr, KeyPurpose::Claim))
    }
}

fn hash_key(env: &Env, key: &Address) -> BytesN<32> {
    let address_bytes = Bytes::from_val(env, &key.to_xdr(&env));
    env.crypto().keccak256(&address_bytes).to_bytes()
}

fn hash_claim(env: &Env, issuer: &Address, topic: &U256) -> BytesN<32> {
    let address_bytes = Bytes::from_val(env, &issuer.to_xdr(&env));
    let topic_bytes = Bytes::from_val(env, &topic.to_xdr(env));

    let mut concatenated_bytes = Bytes::new(env);
    concatenated_bytes.append(&address_bytes);
    concatenated_bytes.append(&topic_bytes);
    env.crypto().keccak256(&concatenated_bytes).to_bytes()
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

#[contracttype]
pub struct Signature {
    pub authenticator_data: Bytes,
    pub client_data_json: Bytes,
    pub signature: BytesN<64>,
}

#[derive(serde::Deserialize)]
struct ClientDataJson<'a> {
    challenge: &'a str,
}

impl CustomAccountInterface for IdentityContract {
    type Signature = Signature;

    type Error = Error;

    fn __check_auth(
        env: Env,
        signature_payload: Hash<32>,
        signature: Self::Signature,
        _auth_contexts: soroban_sdk::Vec<soroban_sdk::auth::Context>,
    ) -> Result<(), Self::Error> {
        let pk = env
            .storage()
            .instance()
            .get(&STORAGE_KEY_PK)
            .ok_or(Error::AlreadyInitialized)?;

        let mut paylaod = Bytes::new(&env);

        paylaod.append(&signature.authenticator_data);
        paylaod.extend_from_array(&env.crypto().sha256(&signature.client_data_json).to_array());

        let payload = env.crypto().sha256(&paylaod);

        env.crypto()
            .secp256r1_verify(&pk, &payload, &signature.signature);

        let client_data_json = signature.client_data_json.to_buffer::<1024>();
        let client_data_json = client_data_json.as_slice();

        let (client_data, _): (ClientDataJson, _) =
            serde_json_core::de::from_slice(client_data_json).map_err(|_| Error::JsonParseError)?;

        let mut expected_challenge = *b"___________________________________________";
        base64_url::encode(&mut expected_challenge, &signature_payload.to_array());

        if client_data.challenge.as_bytes() != expected_challenge {
            return Err(Error::ClientDataJsonChallengeIncorrect);
        }
        Ok(())
    }
}

mod base64_url;
mod test;
