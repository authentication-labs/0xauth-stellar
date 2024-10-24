#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, Address, BytesN, Env, Symbol, Val, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    InsufficientPermissions = 2,
}

#[contract]
pub struct FactoryContract;

#[contractimpl]
impl FactoryContract {
    pub fn get_initialized(env: Env) -> Result<bool, Error> {
        Ok(env
            .storage()
            .instance()
            .get::<Symbol, bool>(&Symbol::new(&env, "initialized"))
            .unwrap_or(false))
    }

    pub fn initialize(env: Env, owner: Address) -> Result<(), Error> {
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

        env.events().publish((init_symbol,), owner);

        Ok(())
    }

    pub fn create_identity(
        env: Env,
        wasm_hash: BytesN<32>,
        wallet: Address,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
    ) -> (Address, Val) {
        let owner = only_owner(&env);

        let identity_address = env.deployer().with_address(owner, salt.clone()).deploy(wasm_hash);

        let res: Val = env.invoke_contract(&identity_address, &init_fn, init_args.clone());

        env.storage().instance().set(&wallet, &identity_address);

        let mut wallets: Vec<Address> = env
            .storage()
            .instance()
            .get::<Address, Vec<Address>>(&identity_address)
            .unwrap_or(Vec::new(&env));

        wallets.push_back(wallet.clone());

        env.storage().instance().set(&identity_address, &wallets);

        env.events().publish((Symbol::new(&env, "create_identity"),), (wallet, identity_address.clone(), salt, init_fn, init_args));

        (identity_address, res)
    }

    pub fn link_wallet(env: Env, wallet: Address, identity: Address) {
        only_owner(&env);

        let mut wallets: Vec<Address> = env
            .storage()
            .instance()
            .get::<Address, Vec<Address>>(&identity)
            .unwrap_or(Vec::new(&env));

        wallets.push_back(wallet.clone());

        env.storage().instance().set(&identity, &wallets);

        env.storage().instance().set(&wallet, &identity);

        env.events().publish((Symbol::new(&env, "link_wallet"),), (wallet, identity));
    }

    pub fn unlink_wallet(env: Env, wallet: Address, identity: Address) {
        only_owner(&env);

        let mut wallets: Vec<Address> = env
            .storage()
            .instance()
            .get::<Address, Vec<Address>>(&identity)
            .unwrap_or(Vec::new(&env));

        if wallets.contains(&wallet) {
            let index = wallets.iter().position(|x| x == wallet).unwrap();
            wallets.remove(index as u32);
        }

        env.storage().instance().set(&identity, &wallets);

        env.storage().instance().remove(&wallet);

        env.events().publish((Symbol::new(&env, "unlink_wallet"),), (wallet, identity));
    }

    pub fn get_wallets(env: Env, identity: Address) -> Vec<Address> {
        let wallets: Vec<Address> = env
            .storage()
            .instance()
            .get::<Address, Vec<Address>>(&identity)
            .unwrap_or(Vec::new(&env));

        wallets
    }

    pub fn get_identity(env: Env, wallet: Address) -> Address {
        let identity: Address = env.storage().instance().get(&wallet).unwrap();
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

        env.events().publish((Symbol::new(&env, "set_owner"),), owner);
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
