use cosmwasm_std::coin;
use cw_orch::{anyhow, prelude::*, tokio};
use lido_satellite::{msg::InstantiateMsg, LidoSatellite};
use tokio::runtime::Runtime;

/// Script that registers the first Account in abstract (our Account)
pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let rt = Runtime::new()?;
    let network = networks::LOCAL_NEUTRON;
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;

    let satellite = LidoSatellite::new("lido_satellite", chain);
    satellite.upload()?;
    satellite.instantiate(
        &InstantiateMsg {
            bridged_denom: "uibcatom".to_string(),
            canonical_subdenom: "wsteth".to_string(),
        },
        None,
        Some(&[coin(1000000, "untrn")]),
    )?;
    Ok(())
}
