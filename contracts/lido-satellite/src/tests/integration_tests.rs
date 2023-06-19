use crate::{
    contract::CONTRACT_NAME,
    msg::{ConfigResponse, InstantiateMsg, QueryMsg},
    LidoSatellite,
};
use cosmwasm_std::Addr;
use cw_orch::prelude::*;

// consts for testing
const ADMIN: &str = "admin";
/// Instantiate the contract in any CosmWasm environment
fn setup<Chain: CwEnv>(chain: Chain) -> LidoSatellite<Chain> {
    // Construct the counter interface
    let contract = LidoSatellite::new(CONTRACT_NAME, chain.clone());
    let admin = Addr::unchecked(ADMIN);

    // Upload the contract
    let upload_resp = contract.upload().unwrap();

    // Get the code-id from the response.
    let code_id = upload_resp.uploaded_code_id().unwrap();
    // or get it from the interface.
    assert_eq!(code_id, contract.code_id().unwrap());

    // Instantiate the contract
    let msg = InstantiateMsg {
        bridged_denom: "".to_string(),
        canonical_subdenom: "".to_string(),
    };
    let init_resp = contract.instantiate(&msg, Some(&admin), None).unwrap();

    // Get the address from the response
    let contract_addr = init_resp.instantiated_contract_address().unwrap();
    // or get it from the interface.
    assert_eq!(contract_addr, contract.address().unwrap());

    // Return the interface
    contract
}

#[test]
fn config() {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Set up the contract
    let contract = setup(mock.clone());

    // Perform test
    let config: ConfigResponse = contract.query(&QueryMsg::Config {}).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            bridged_denom: "".to_string(),
            canonical_subdenom: "".to_string(),
        }
    );
}
