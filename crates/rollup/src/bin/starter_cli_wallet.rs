//! This binary defines a cli wallet for interacting
//! with the rollup.

use sov_modules_api::cli::{FileNameArg, JsonStringArg};
use sov_modules_rollup_blueprint::WalletBlueprint;
#[cfg(all(feature = "celestia_da", not(feature = "mock_da")))]
use sov_rollup_starter::celestia_rollup::CelestiaRollup as StarterRollup;
#[cfg(all(feature = "mock_da", not(feature = "celestia_da")))]
use sov_rollup_starter::mock_rollup::MockRollup as StarterRollup;
use stf_starter::runtime::RuntimeSubcommand;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    StarterRollup::run_wallet::<
        RuntimeSubcommand<FileNameArg, _>,
        RuntimeSubcommand<JsonStringArg, _>,
    >()
    .await
}
