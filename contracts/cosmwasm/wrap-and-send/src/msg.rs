use astroport::router::SwapOperation;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Uint128};
use neutron_sdk::bindings::msg::IbcFee;

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of Lido Satellite contract, used to mint canonical funds
    pub lido_satellite: String,
    /// Address of Astroport Router contract, used to swap canonical funds into IBC fee funds
    pub astroport_router: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// This method expects user to send bridged funds, which will be sent to Lido Satellite
    /// and locked in there. Lido Satellite will mint canonical funds in return, which will
    /// be sent further to another destination chain. This call also automatically swaps a small
    /// part of funds for IBC fee denom in order to pay for IBC fee. In case of any failure
    /// possible, all funds will be refunded to specified refund address controlled by user
    WrapAndSend {
        /// Source port to send funds from
        source_port: String,
        /// Source channel to send funds from
        source_channel: String,
        /// Address of the receiver on a remote chain
        receiver: String,
        /// Amount of bridged funds which will be swapped for IBC fee denom. If supplied amount of
        /// bridged funds is less than this value, user will receive canonical funds on their
        /// specified refund address. This amount should cover both `ack_fee` and `timeout_fee`,
        /// unused amount of fee will be refunded as well
        amount_to_swap_for_ibc_fee: Uint128,
        /// Denom used to spend on IBC fee. It will be acquired from Astroport Router in exchange
        /// for `amount_to_swap_for_ibc_fee` amount of canonical funds
        ibc_fee_denom: String,
        /// Array of swap instructions for Astroport router. These instructions MUST swap from
        /// canonical denom to IBC fee denom, otherwise transaction will revert and user will
        /// receive canonical funds and result of swap on their specified refund address
        astroport_swap_operations: Vec<SwapOperation>,
        /// Address of user account on Neutron network which will be used for all refunds
        refund_address: String,
    },
    /// Internal call, only contract itself can execute it. Users of a contract shall ignore and
    /// never try to use it.
    WrapCallback {
        source_port: String,
        source_channel: String,
        receiver: String,
        amount_to_swap_for_ibc_fee: Uint128,
        ibc_fee_denom: String,
        astroport_swap_operations: Vec<SwapOperation>,
        received_amount: Uint128,
        refund_address: Addr,
    },
    /// Internal call, only contract itself can execute it. Users of a contract shall ignore and
    /// never try to use it.
    SwapCallback {
        source_port: String,
        source_channel: String,
        receiver: String,
        amount_to_send: Coin,
        min_ibc_fee: IbcFee,
        refund_address: Addr,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub lido_satellite: String,
    pub astroport_router: String,
    pub bridged_denom: String,
    pub canonical_denom: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
