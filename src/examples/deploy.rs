// This is a failed attempt to write integration tests using cw-orch
// TODO: try again when https://github.com/AbstractSDK/cw-orchestrator/issues/143
//       is resolved

/*use cw_orch::{anyhow, prelude::*, tokio};
use lido_satellite::msg::InstantiateMsg;
use tokio::runtime::Runtime;

/// Script that registers the first Account in abstract (our Account)
pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let rt = Runtime::new()?;
    let network = networks::LOCAL_JUNO;
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;

    let counter = CounterContract::new("counter_contract", chain);

    counter.upload()?;
    counter.instantiate(&InstantiateMsg { count: 0 }, None, None)?;
    Ok(())
}
*/
pub fn main() {}
