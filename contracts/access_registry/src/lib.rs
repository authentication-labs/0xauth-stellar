#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, vec, Address, Env, String, Symbol, Vec,
};

#[contract]
pub struct AccessRegistryContract;

const INIT_SYMB: Symbol = symbol_short!("INIT");
const FUND_ID: Symbol = symbol_short!("FUND_ID");
const ADMIN: Symbol = symbol_short!("ADMIN");
const ADDRESSES: Symbol = symbol_short!("ADDRS");

#[contractimpl]
impl AccessRegistryContract {
    pub fn initialize(env: Env, fund_id: String, admin: Address) -> Result<(), Error> {
        if env
            .storage()
            .instance()
            .get::<Symbol, bool>(&INIT_SYMB)
            .unwrap_or(false)
        {
            return Err(Error::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set::<Symbol, String>(&FUND_ID, &fund_id);

        env.storage()
            .instance()
            .set::<Symbol, Address>(&ADMIN, &admin);

        env.storage()
            .instance()
            .set::<Symbol, bool>(&INIT_SYMB, &true);

        return Ok(());
    }
    pub fn get_initialized(env: Env) -> Result<bool, Error> {
        Ok(env
            .storage()
            .instance()
            .get::<Symbol, bool>(&INIT_SYMB)
            .unwrap_or(false))
    }
    pub fn get_fundid(env: Env) -> Option<String> {
        return env.storage().instance().get(&FUND_ID);
    }

    pub fn get_admin(env: Env) -> Option<Address> {
        return env.storage().instance().get(&ADMIN);
    }

    pub fn add_address(env: Env, admin: Address, addr: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;

        // get address
        let mut addresses: Vec<Address> = env
            .storage()
            .instance()
            .get::<Symbol, Vec<Address>>(&ADDRESSES)
            .unwrap_or(Vec::new(&env));

        if addresses.contains(&addr) {
            return Ok(());
        }

        addresses.push_back(addr);

        env.storage()
            .instance()
            .set::<Symbol, Vec<Address>>(&ADDRESSES, &addresses);

        return Ok(());
    }
    pub fn remove_address(env: Env, admin: Address, addr: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        // get addresses
        let mut addresses: Vec<Address> = env
            .storage()
            .instance()
            .get::<Symbol, Vec<Address>>(&ADDRESSES)
            .unwrap_or(Vec::new(&env));

        // get pos and remove
        if let Some(pos) = addresses.iter().position(|a| a == addr) {
            addresses.remove(pos as u32);
        }

        // save updated
        env.storage()
            .instance()
            .set::<Symbol, Vec<Address>>(&ADDRESSES, &addresses);

        return Ok(());
    }
    pub fn get_addresses(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get::<Symbol, Vec<Address>>(&ADDRESSES)
            .unwrap_or(Vec::new(&env))
    }
    pub fn is_whitelisted(env: Env, addr: Address) -> bool {
        env.storage()
            .instance()
            .get::<Symbol, Vec<Address>>(&ADDRESSES)
            .unwrap_or(Vec::new(&env))
            .contains(addr)
    }
}
fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    let admin = env
        .storage()
        .instance()
        .get::<Symbol, Address>(&ADMIN)
        .ok_or(Error::Unauthorized)?;

    if admin != *caller {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    Unauthorized = 2,
}

mod test;
