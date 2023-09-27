use crate::{state::CONFIG, ContractError, ContractResult};
use astroport::router::SwapOperation;
use cosmwasm_std::{
    coins, BankMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdError, Uint128,
};

pub fn execute_swap_operations(
    deps: &DepsMut,
    _env: Env,
    mut info: MessageInfo,
    mut operations: Vec<SwapOperation>,
    minimum_receive: Option<Uint128>,
    to: Option<String>,
    max_spread: Option<Decimal>,
) -> ContractResult<Response> {
    deps.api
        .debug(&format!("WASMDEBUG: operations: {:?}", &operations));
    deps.api.debug(&format!(
        "WASMDEBUG: minimum_receive: {:?}",
        &minimum_receive
    ));
    deps.api.debug(&format!("WASMDEBUG: info: {:?}", &info));

    let config = CONFIG.load(deps.storage)?;

    assert_eq!(operations.len(), 1);
    // value of 2000untrn is enough to launch one IBC transfer on Neutron localnet
    assert_eq!(minimum_receive, Some(Uint128::new(2000)));
    assert!(to.is_none());
    assert!(max_spread.is_none());
    deps.api.debug("WASMDEBUG: initial asserts passed");

    let operation = operations.pop().unwrap();
    deps.api.debug("WASMDEBUG: operation extracted");
    match operation {
        SwapOperation::NativeSwap {
            offer_denom,
            ask_denom,
        } => {
            deps.api.debug("WASMDEBUG: 0");
            assert_eq!(offer_denom, config.offer_denom);
            deps.api.debug("WASMDEBUG: 1");
            assert_eq!(ask_denom, config.ask_denom);
            deps.api.debug("WASMDEBUG: 2");
            assert_eq!(info.funds.len(), 1);
            deps.api.debug("WASMDEBUG: secondary asserts passed");

            let funds = info.funds.pop().unwrap();
            deps.api.debug("WASMDEBUG: funds extracted");
            assert_eq!(funds.denom, config.offer_denom);
            deps.api.debug("WASMDEBUG: final asserts passed");
            match funds.amount.u128() {
                100 => {
                    // swap fails
                    Err(ContractError::Std(StdError::generic_err("swap fails")))
                }
                200 => {
                    // swap returns less than minimum_receive
                    Ok(Response::new().add_message(BankMsg::Send {
                        to_address: info.sender.to_string(),
                        amount: coins(1250, config.ask_denom),
                    }))
                }
                300 => {
                    // swap returns exactly minimum receive
                    Ok(Response::new().add_message(BankMsg::Send {
                        to_address: info.sender.to_string(),
                        amount: coins(2000, config.ask_denom),
                    }))
                }
                400 => {
                    // swap returns more than minimum receive
                    Ok(Response::new().add_message(BankMsg::Send {
                        to_address: info.sender.to_string(),
                        amount: coins(2334, config.ask_denom),
                    }))
                }
                _ => unimplemented!(),
            }
        }
        SwapOperation::AstroSwap { .. } => unimplemented!(),
    }
}
