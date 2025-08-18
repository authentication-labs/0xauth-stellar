#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    Symbol, Val, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    InsufficientPermissions = 2,
    NotInitiated = 3,
}

#[contract]
pub struct FactoryContract;

const STORAGE_KEY_WASM_HASH: Symbol = symbol_short!("hash");

#[contractimpl]
impl FactoryContract {
    pub fn extend_ttl(env: Env) {
        let max_ttl = env.storage().max_ttl();
        let contract_address = env.current_contract_address();

        env.storage().instance().extend_ttl(max_ttl, max_ttl);
        env.deployer()
            .extend_ttl(contract_address.clone(), max_ttl, max_ttl);
        env.deployer()
            .extend_ttl_for_code(contract_address.clone(), max_ttl, max_ttl);
        env.deployer()
            .extend_ttl_for_contract_instance(contract_address.clone(), max_ttl, max_ttl);
    }

    pub fn get_initialized(env: Env) -> Result<bool, Error> {
        Ok(env
            .storage()
            .instance()
            .get::<Symbol, bool>(&Symbol::new(&env, "initialized"))
            .unwrap_or(false))
    }

    pub fn initialize(env: Env, owner: Address, wasm_hash: BytesN<32>) -> Result<(), Error> {
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

        env.storage()
            .instance()
            .set(&symbol_short!("owner"), &owner);

        env.storage()
            .instance()
            .set(&STORAGE_KEY_WASM_HASH, &wasm_hash);

        env.events().publish((init_symbol,), owner);

        Self::extend_ttl(env);
        Ok(())
    }

    pub fn create_identity(
        env: Env,
        pub_key: BytesN<65>,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
    ) -> Result<(Address, Val), Error> {
        let owner = only_owner(&env);

        let wasm_hash = env
            .storage()
            .instance()
            .get::<Symbol, BytesN<32>>(&STORAGE_KEY_WASM_HASH)
            .ok_or(Error::NotInitiated)?;

        let identity_address = env
            .deployer()
            .with_address(owner, salt.clone())
            .deploy_v2(wasm_hash, ());

        let res: Val = env.invoke_contract(&identity_address, &init_fn, init_args.clone());

        env.storage().instance().set(&pub_key, &identity_address);

        let mut pub_keys = env
            .storage()
            .instance()
            .get::<Address, Vec<BytesN<65>>>(&identity_address)
            .unwrap_or(Vec::new(&env));

        pub_keys.push_back(pub_key.clone());

        env.storage().instance().set(&identity_address, &pub_keys);

        env.events().publish(
            (Symbol::new(&env, "create_identity"),),
            (pub_key, identity_address.clone(), salt, init_fn, init_args),
        );

        Ok((identity_address, res))
    }

    pub fn link_pubkey(env: Env, pub_key: BytesN<65>, identity: Address) {
        only_owner(&env);

        let mut pub_keys = env
            .storage()
            .instance()
            .get::<Address, Vec<BytesN<65>>>(&identity)
            .unwrap_or(Vec::new(&env));

        pub_keys.push_back(pub_key.clone());

        env.storage().instance().set(&identity, &pub_keys);

        env.storage().instance().set(&pub_key, &identity);

        env.events()
            .publish((Symbol::new(&env, "link_pk"),), (pub_key, identity));
    }

    pub fn unlink_pubkey(env: Env, pub_key: BytesN<65>, identity: Address) {
        only_owner(&env);

        let mut pub_keys = env
            .storage()
            .instance()
            .get::<Address, Vec<BytesN<65>>>(&identity)
            .unwrap_or(Vec::new(&env));

        if pub_keys.contains(&pub_key) {
            let index = pub_keys.iter().position(|x| x == pub_key).unwrap();
            pub_keys.remove(index as u32);
        }

        env.storage().instance().set(&identity, &pub_keys);

        env.storage().instance().remove(&pub_keys);

        env.events()
            .publish((Symbol::new(&env, "unlink_pk"),), (pub_key, identity));
    }

    pub fn get_pks(env: Env, identity: Address) -> Vec<Address> {
        let pub_keys: Vec<Address> = env
            .storage()
            .instance()
            .get::<Address, Vec<Address>>(&identity)
            .unwrap_or(Vec::new(&env));

        pub_keys
    }

    pub fn get_identity(env: Env, pk: BytesN<65>) -> Address {
        let identity: Address = env.storage().instance().get(&pk).unwrap();
        identity
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

        env.events()
            .publish((Symbol::new(&env, "set_owner"),), owner);
    }
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
