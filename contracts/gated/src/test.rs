#![cfg(test)]

use crate::{GatedContract, GatedContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};
extern crate std;

use crate::claim_issuer;
use crate::factory;

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let gated_contract_id = env.register_contract(None, GatedContract);
    let gated_client = GatedContractClient::new(&env, &gated_contract_id);

    let factory_contract_id = env.register_contract_wasm(None, factory::WASM);
    let claim_issuer_contract_id = env.register_contract_wasm(None, claim_issuer::WASM);

    let owner = Address::generate(&env);
    gated_client.initialize(&factory_contract_id, &claim_issuer_contract_id, &owner);

    assert_eq!(
        gated_client.get_initialized(),
        true,
        "Contract should be initialized"
    );
}
