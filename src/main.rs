use alloy_network::EthereumWallet;
use alloy_primitives::{address, Bytes, FixedBytes};
use alloy_provider::ProviderBuilder;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProofFixture {
    public_values: Bytes,
    proof: Bytes,
}

sol! {
    #[sol(rpc)] // <-- Important! Generates the necessary `SnarkBasedFeeOracle` struct and function methods.
    contract SnarkBasedFeeOracle {
        constructor(address) {} // The `deploy` method will also include any constructor arguments.

        #[derive(Debug)]
        function verifyAndUpdate(bytes proof, bytes public_values);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Need a private key for signing the transaction
    let private_key = env::var("PRIVATE_KEY")?;
    let drpc_key = env::var("DRPC_KEY")?;
    let drpc_url = format!(
        "https://lb.drpc.org/ogrpc?network=sepolia&dkey={}",
        drpc_key
    );
    let signer = PrivateKeySigner::from_bytes(&FixedBytes::from_str(&private_key)?)?;
    let wallet = EthereumWallet::new(signer);

    // Build a provider.
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_builtin(&drpc_url)
        .await?;

    // Create a new contract instance can be created with `SnarkBasedFeeOracle::new`.
    let address = address!("6Ae1e19F65b474B7Eff9A22F33cc72611b0FC24A");
    let contract = SnarkBasedFeeOracle::new(address, &provider);

    // Build a call to the `verifyAndUpdate` function and configure it.
    let proof_string = std::fs::read_to_string("./src/fixture.json")?;
    let proof_fixture: ProofFixture = serde_json::from_str(&proof_string)?;
    let proof = proof_fixture.proof;
    let public_values = proof_fixture.public_values;
    let call_builder = contract.verifyAndUpdate(proof, public_values);

    // Send the call. Note that this is not broadcasted as a transaction.
    let call_return = call_builder.call().await?;
    println!("{call_return:?}"); // verifyAndUpdateReturn

    // Use `send` to broadcast the call as a transaction.
    let _pending_tx = call_builder.send().await?;
    Ok(())
}
