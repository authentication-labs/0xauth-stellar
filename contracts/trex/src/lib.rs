#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, Address, Env, Symbol, U256, token
};

mod gated {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/gated.wasm"
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
pub struct TrexContract;

#[contractimpl]
impl TrexContract {
    pub fn get_initialized(env: Env) -> Result<bool, Error> {
        Ok(env
            .storage()
            .instance()
            .get::<Symbol, bool>(&Symbol::new(&env, "initialized"))
            .unwrap_or(false))
    }

    pub fn initialize(
        env: Env,
        gated_address: Address,
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
            .instance()
            .set(&symbol_short!("owner"), &owner);
        env.storage()
            .instance()
            .set(&symbol_short!("gated"), &gated_address);

        Ok(())
    }
    pub fn transfer(env: Env, id: Address, from: Address, to: Address, amount: i128) -> Result<bool, Error> {
        let token_client = token::Client::new(&env, &id);

        let gated_address: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("gated"))
            .unwrap();

        let issuer: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("issuer"))
            .unwrap();

        let gated_client = gated::Client::new(&env, &gated_address);
        if !gated_client.validate_claim(&from, &issuer, &U256::from_u128(&env, 1)) {
            return Err(Error::InsufficientPermissions);
        }

        token_client.transfer(&from, &to, &amount);

        return Ok(true);
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


fn only_owner(env: &Env) -> Address {
    let owner: Address = env
        .storage()
        .instance()
        .get(&symbol_short!("owner"))
        .unwrap();
    owner.require_auth();

    owner
}
