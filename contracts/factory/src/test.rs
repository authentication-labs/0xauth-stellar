#![cfg(test)]
extern crate std;

use super::*;
use secp256k1::hashes::{sha256, Hash};
use secp256k1::rand;
use secp256k1::{Message, Secp256k1};

use soroban_sdk::{testutils::Address as _, Address, Env};

mod identity {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/identity.wasm");
}

#[test]
fn test_linking() {
    let env = Env::default();
    env.mock_all_auths();

    let factory_contract_id = env.register(FactoryContract, ());
    let factory_client = FactoryContractClient::new(&env, &factory_contract_id);

    let owner = Address::generate(&env);
    let identity_id = Address::generate(&env);

    let id_wasm = env.deployer().upload_contract_wasm(identity::WASM);

    let secp = Secp256k1::new();
    let (sec_key, pk) = secp.generate_keypair(&mut rand::rng());

    let pk: BytesN<65> = BytesN::from_array(&env, &pk.serialize_uncompressed());
    // Initialize the factory contract
    factory_client.initialize(&owner, &id_wasm);

    // Link the wallet to the identity
    factory_client.link_pubkey(&pk, &identity_id);

    // Get the identity
    let linked_identity = factory_client.get_identity(&pk);

    assert!(
        linked_identity == identity_id,
        "Identity should be linked to wallet"
    );
}
