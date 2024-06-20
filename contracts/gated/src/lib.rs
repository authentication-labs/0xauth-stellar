#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, Address, BytesN, Bytes, Env, Symbol, U256,FromVal, xdr::ToXdr
};

mod factory {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/factory.wasm"
    );
}
mod claim_issuer {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/claim_issuer.wasm"
    );
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    InsufficientPermissions = 2,
    IdentityNotFound = 3,
}

#[contract]
pub struct GatedContract;

#[contractimpl]
impl GatedContract {
    pub fn get_initialized(env: Env) -> Result<bool, Error> {
        Ok(env
            .storage()
            .instance()
            .get::<Symbol, bool>(&Symbol::new(&env, "initialized"))
            .unwrap_or(false))
    }

    pub fn initialize(
        env: Env,
        identity_factory: Address,
        claim_issuer: Address,
        owner: Address,
    ) -> Result<(), Error> {
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
        env.storage()
            .persistent()
            .set(&symbol_short!("factory"), &identity_factory);
        env.storage()
            .persistent()
            .set(&symbol_short!("issuer"), &claim_issuer);
        env.storage()
            .instance()
            .set(&symbol_short!("owner"), &owner);

        Ok(())
    }


    pub fn set_claim_issuer(env: Env, issuer: Address) {
        only_owner(&env);

        env.storage().persistent().set(&symbol_short!("issuer"), &issuer);
    }

    pub fn set_identity_factory(env: Env, factory: Address) {
        only_owner(&env);

        env.storage().persistent().set(&symbol_short!("factory"), &factory);
    }

    pub fn validate_claim(env: Env, sender: Address,  issuer: Address, required_topic: U256) -> bool {
        let factory_address = env.storage()
        .persistent()
            .get(&symbol_short!("factory"))
            .unwrap();

        let factory_client = factory::Client::new(&env, &factory_address);
        let user_identity = factory_client.get_identity(&sender);
        let claim_id = hash_claim(&env, &issuer, &required_topic);
        let issuer_client = claim_issuer::Client::new(&env, &user_identity);
        let claim = issuer_client.get_claim(&claim_id).unwrap();
        return  issuer_client.is_claim_valid(&issuer, &claim.issuer, &claim.topic, &claim.signature, &claim.data);

    }

    pub fn get_owner(env: Env) -> Address {
        let owner: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("owner"))
            .unwrap();
        owner
    }

    pub fn set_owner(env: Env, owner: Address) {
        only_owner(&env);

        env.storage()
            .instance()
            .set(&symbol_short!("owner"), &owner);
    }
}

fn hash_claim(env: &Env, issuer: &Address, topic: &U256) -> BytesN<32> {
    let address_bytes = Bytes::from_val(env, &issuer.to_xdr(&env));
    let topic_bytes = Bytes::from_val(env, &topic.to_xdr(env));

    let mut concatenated_bytes = Bytes::new(env);
    concatenated_bytes.append(&address_bytes);
    concatenated_bytes.append(&topic_bytes);
    env.crypto().keccak256(&concatenated_bytes)
}

fn only_owner(env: &Env) -> Address {
    let owner: Address = env
        .storage()
        .instance()
        .get(&symbol_short!("owner"))
        .unwrap();
    owner.require_auth();

    owner
}


mod test;