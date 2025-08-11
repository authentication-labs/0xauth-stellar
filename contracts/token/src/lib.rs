#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

const DECIMAL: u32 = 7;
const NAME: Symbol = symbol_short!("MyToken");
const SYMBOL: Symbol = symbol_short!("MTK");

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Balance(Address),
    TotalSupply,
    Allowance(Address, Address),
}

#[derive(Clone)]
#[contracttype]
pub struct TokenMetadata {
    pub decimal: u32,
    pub name: String,
    pub symbol: String,
}

pub trait TokenTrait {
    fn initialize(env: Env, admin: Address, decimal: u32, name: String, symbol: String);

    fn mint(env: Env, to: Address, amount: i128);

    fn burn(env: Env, from: Address, amount: i128);

    fn transfer(env: Env, from: Address, to: Address, amount: i128);

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128);

    fn approve(env: Env, from: Address, spender: Address, amount: i128);

    fn balance(env: Env, id: Address) -> i128;

    fn allowance(env: Env, from: Address, spender: Address) -> i128;

    fn decimals(env: Env) -> u32;

    fn name(env: Env) -> String;

    fn symbol(env: Env) -> String;

    fn total_supply(env: Env) -> i128;

    fn set_admin(env: Env, new_admin: Address);

    fn admin(env: Env) -> Address;
}

#[contract]
pub struct Token;

#[contractimpl]
impl TokenTrait for Token {
    fn initialize(env: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);

        let metadata = TokenMetadata {
            decimal,
            name,
            symbol,
        };

        env.storage()
            .instance()
            .set(&symbol_short!("METADATA"), &metadata);
    }

    fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if amount < 0 {
            panic!("Amount cannot be negative");
        }

        let balance = Self::balance(env.clone(), to.clone());
        let new_balance = balance + amount;

        let total_supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        let new_total_supply = total_supply + amount;

        env.storage()
            .instance()
            .set(&DataKey::Balance(to.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_total_supply);

        env.events().publish((symbol_short!("mint"), &to), amount);
    }

    fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();

        if amount < 0 {
            panic!("Amount cannot be negative");
        }

        let balance = Self::balance(env.clone(), from.clone());
        if balance < amount {
            panic!("Insufficient balance");
        }

        let new_balance = balance - amount;
        let total_supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        let new_total_supply = total_supply - amount;

        env.storage()
            .instance()
            .set(&DataKey::Balance(from.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_total_supply);

        env.events().publish((symbol_short!("burn"), &from), amount);
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        if amount < 0 {
            panic!("Amount cannot be negative");
        }

        let from_balance = Self::balance(env.clone(), from.clone());
        if from_balance < amount {
            panic!("Insufficient balance");
        }

        let to_balance = Self::balance(env.clone(), to.clone());

        env.storage()
            .instance()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .instance()
            .set(&DataKey::Balance(to.clone()), &(to_balance + amount));

        env.events()
            .publish((symbol_short!("transfer"), &from, &to), amount);
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        if amount < 0 {
            panic!("Amount cannot be negative");
        }

        let allowance = Self::allowance(env.clone(), from.clone(), spender.clone());
        if allowance < amount {
            panic!("Insufficient allowance");
        }

        let from_balance = Self::balance(env.clone(), from.clone());
        if from_balance < amount {
            panic!("Insufficient balance");
        }

        let to_balance = Self::balance(env.clone(), to.clone());

        env.storage()
            .instance()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .instance()
            .set(&DataKey::Balance(to.clone()), &(to_balance + amount));
        env.storage().instance().set(
            &DataKey::Allowance(from.clone(), spender.clone()),
            &(allowance - amount),
        );

        env.events()
            .publish((symbol_short!("transfer"), &from, &to), amount);
    }

    fn approve(env: Env, from: Address, spender: Address, amount: i128) {
        from.require_auth();

        if amount < 0 {
            panic!("Amount cannot be negative");
        }

        env.storage()
            .instance()
            .set(&DataKey::Allowance(from.clone(), spender.clone()), &amount);

        env.events()
            .publish((symbol_short!("approve"), &from, &spender), amount);
    }

    fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Balance(id))
            .unwrap_or(0)
    }

    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Allowance(from, spender))
            .unwrap_or(0)
    }

    fn decimals(env: Env) -> u32 {
        let metadata: TokenMetadata = env
            .storage()
            .instance()
            .get(&symbol_short!("METADATA"))
            .unwrap_or(TokenMetadata {
                decimal: DECIMAL,
                name: String::from_str(&env, "MyToken"),
                symbol: String::from_str(&env, "MTK"),
            });
        metadata.decimal
    }

    fn name(env: Env) -> String {
        let metadata: TokenMetadata = env
            .storage()
            .instance()
            .get(&symbol_short!("METADATA"))
            .unwrap_or(TokenMetadata {
                decimal: DECIMAL,
                name: String::from_str(&env, "MyToken"),
                symbol: String::from_str(&env, "MTK"),
            });
        metadata.name
    }

    fn symbol(env: Env) -> String {
        let metadata: TokenMetadata = env
            .storage()
            .instance()
            .get(&symbol_short!("METADATA"))
            .unwrap_or(TokenMetadata {
                decimal: DECIMAL,
                name: String::from_str(&env, "MyToken"),
                symbol: String::from_str(&env, "MTK"),
            });
        metadata.symbol
    }

    fn total_supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    fn set_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        env.events()
            .publish((symbol_short!("set_admin"),), &new_admin);
    }

    fn admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }
}
