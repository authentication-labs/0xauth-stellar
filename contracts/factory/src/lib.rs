#![no_std]
use soroban_sdk::{contract, contractimpl, contracterror, symbol_short, Env, Symbol, Vec, Address, BytesN, Val};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    InsufficientPermissions = 2
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
        owner.require_auth();

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

        Ok(())
    }

    pub fn create_identity(
        env: Env,
        deployer: Address,
        wasm_hash: BytesN<32>,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
    ) -> (Address, Val) {
        if deployer != env.current_contract_address() {
            deployer.require_auth();
        } else {
            only_owner(&env);
        }

        let deployed_address = env
            .deployer()
            .with_address(deployer, salt)
            .deploy(wasm_hash);


        let res: Val = env.invoke_contract(&deployed_address, &init_fn, init_args);


        (deployed_address, res)
    }
}

fn only_owner(env: &Env) {
    let owner: Address = env
        .storage()
        .instance()
        .get(&symbol_short!("owner"))
        .unwrap();
    owner.require_auth();
}

mod test;
