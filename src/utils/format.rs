use anyhow::Context;
use serde_json::{Value, json};
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;
use yellowstone_grpc_proto::{convert_from, prelude::SubscribeUpdateTransactionInfo};

pub fn create_pretty_transaction(tx: SubscribeUpdateTransactionInfo) -> anyhow::Result<Value> {
    Ok(json!({
        "signature": Signature::try_from(tx.signature.as_slice()).context("invalid signature")?.to_string(),
        "isVote": tx.is_vote,
        "tx": convert_from::create_tx_with_meta(tx)
            .map_err(|error| anyhow::anyhow!(error))
            .context("invalid tx with meta")?
            .encode(UiTransactionEncoding::Base64, Some(u8::MAX), true)
            .context("failed to encode transaction")?,
    }))
}
