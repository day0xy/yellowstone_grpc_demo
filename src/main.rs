mod client;
mod filters;
mod types;
mod utils;

use anyhow::Context;
use chrono::{Local, Utc};
use dotenvy::dotenv;
use futures::{sink::SinkExt, stream::StreamExt};
use log::info;
use serde_json::json;
use std::io::Write;
use std::{collections::HashMap, env};
use tokio::time::{Duration, interval};

use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_proto::{
    geyser::SlotStatus,
    prelude::{
        CommitmentLevel, SubscribeRequest, SubscribeRequestFilterSlots, SubscribeRequestPing,
        SubscribeUpdatePong, SubscribeUpdateSlot, subscribe_update::UpdateOneof,
    },
};

use client::connection::GrpcClient;
use filters::{new_filter_accounts, new_filter_transactions};
use types::pump_fun::{CreateEvent, EventTrait};
use utils::format::{create_pretty_account, create_pretty_transaction};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    env_logger::Builder::from_default_env()
        .format(move |buf, record| {
            // let time_str = Local::now().format("%Y-%m-%d %H:%M:%S UTC+8").to_string();

            let time_str = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

            writeln!(
                buf,
                "[{} {} {}] {}",
                time_str,
                record.level(),
                record.file().unwrap_or("unknown"),
                record.args()
            )
        })
        .init();

    let endpoint = env::var("YELLOWSTONE_GRPC_URL")?;
    let grpc = GrpcClient::new(endpoint, None);
    // let account_include = vec!["39H3DGBpHpffjTuwQDR9yv9AgbK4U4hesLdsVZ9yDDc9".to_string()];
    let account_include = vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string()];
    let mut client = grpc.build_client().await?;

    let request = new_filter_transactions(account_include, None, None);
    // let request = new_filter_accounts(Some(account_include), None);

    //test slot filter
    // let request = SubscribeRequest {
    //     slots: HashMap::from([(
    //         "client".to_string(),
    //         SubscribeRequestFilterSlots {
    //             filter_by_commitment: Some(true),
    //             interslot_updates: Some(false),
    //         },
    //     )]),
    //     commitment: Some(CommitmentLevel::Processed as i32),
    //     ..Default::default()
    // };

    let (mut subscribe_tx, mut stream) = client.subscribe_with_request(Some(request)).await?;

    while let Some(message) = stream.next().await {
        match message?.update_oneof.expect("invalid message") {
            UpdateOneof::Account(msg) => {
                let account = msg
                    .account
                    .ok_or(anyhow::anyhow!("no account in the message"))?;
                let mut value = create_pretty_account(account)?;
                value["isStartup"] = json!(msg.is_startup);
                value["slot"] = json!(msg.slot);
                info!(
                    "Receive Account: {}",
                    serde_json::to_string(&value).expect("json serialization failed")
                );
            }
            UpdateOneof::Transaction(msg) => {
                if let Some(meta) = msg.transaction.clone().and_then(|t| t.meta) {
                    let logs = meta.log_messages;
                    if let Some(create_event) = CreateEvent::parse_logs::<CreateEvent>(&logs) {
                        info!("detect creating token: {:#?}", create_event);
                    };
                }

                // let mut value = create_pretty_transaction(tx)?;
                // value["slot"] = json!(msg.slot);
                // info!(
                //     "Receive transaction: {}",
                //     serde_json::to_string(&value).expect("json serialization failed")
                // );
            }
            UpdateOneof::Slot(msg) => {
                let status =
                    SlotStatus::try_from(msg.status).context("failed to decode commitment")?;
                let value = json!({
                    "slot": msg.slot,
                    "parent": msg.parent,
                    "status": status.as_str_name(),
                    "deadError": msg.dead_error,
                });
                info!(
                    "Receive Slot: {}",
                    serde_json::to_string(&value).expect("json serialization failed")
                );
            }
            UpdateOneof::Ping(_) => {
                let _ = subscribe_tx
                    .send(SubscribeRequest {
                        ping: Some(SubscribeRequestPing { id: 1 }),
                        ..Default::default()
                    })
                    .await;
                // info!("service is ping: {:#?}", Local::now());
                info!("service is ping");
            }
            UpdateOneof::Pong(SubscribeUpdatePong { id }) => {
                info!("pong received id{id}")
            }
            msg => anyhow::bail!("receive unexpected message: {msg:?}"),
        }
    }

    Ok::<(), anyhow::Error>(())

    // let logs = vec![
    //     "Program ComputeBudget111111111111111111111111111111 invoke [1]".to_string(),
    //     "Program ComputeBudget111111111111111111111111111111 success".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [1]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P invoke [1]".to_string(),
    //     "Program log: Instruction: Create".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [2]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
    //     "Program log: Instruction: InitializeMint2".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2780 of 258300 compute units".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [2]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [2]".to_string(),
    //     "Program log: Create".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]".to_string(),
    //     "Program log: Instruction: GetAccountDataSize".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1595 of 237728 compute units".to_string(),
    //     "Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [3]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program log: Initialize the associated token account".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]".to_string(),
    //     "Program log: Instruction: InitializeImmutableOwner".to_string(),
    //     "Program log: Please upgrade to SPL Token 2022 for immutable owner support".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 231115 compute units".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]".to_string(),
    //     "Program log: Instruction: InitializeAccount3".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4214 of 227231 compute units".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
    //     "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 20490 of 243203 compute units".to_string(),
    //     "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success".to_string(),
    //     "Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s invoke [2]".to_string(),
    //     "Program log: IX: Create Metadata Accounts v3".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [3]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program log: Allocate space for the account".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [3]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program log: Assign the account to the owning program".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [3]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s consumed 39674 of 208073 compute units".to_string(),
    //     "Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s success".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
    //     "Program log: Instruction: MintTo".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4492 of 165781 compute units".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
    //     "Program log: Instruction: SetAuthority".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2911 of 159058 compute units".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
    //     "Program data: G3KpTd7rY3YeAAAAZnVsbHkgYXV0aXN0aWMgcmV0YXJkcyB0cmFkaW5nBAAAAGZhcnRDAAAAaHR0cHM6Ly9pcGZzLmlvL2lwZnMvUW1TMXJjVTFWdlBUVnB1SnFXclRrYzVnNkM5R05xYmlIenY4MjRWaW1XR0tnUBhzl4PXzOleE1tsweuIRUdj/NSS8IXR60VWb59eum+Y5XXOxUjcNAcYU2PJsE48P5j3xbZRTPkCPTz/yj73g9qoTpfLJUgat5sqB+S0hUDriQGAz5X4OaQOuCUGItBQQKhOl8slSBq3myoH5LSFQOuJAYDPlfg5pA64JQYi0FBAZqZgaAAAAAAAENhH488DAACsI/wGAAAAAHjF+1HRAgAAgMakfo0DAA==".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P invoke [2]".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P consumed 2006 of 149925 compute units".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P success".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P consumed 122644 of 269700 compute units".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P success".to_string(),
    //     "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]".to_string(),
    //     "Program log: CreateIdempotent".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
    //     "Program log: Instruction: GetAccountDataSize".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1569 of 141657 compute units".to_string(),
    //     "Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [2]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program log: Initialize the associated token account".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
    //     "Program log: Instruction: InitializeImmutableOwner".to_string(),
    //     "Program log: Please upgrade to SPL Token 2022 for immutable owner support".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 135071 compute units".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
    //     "Program log: Instruction: InitializeAccount3".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4188 of 131192 compute units".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
    //     "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 20335 of 147056 compute units".to_string(),
    //     "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success".to_string(),
    //     "Program FAdo9NCw1ssek6Z6yeWzWjhLVsr8uiCwcWNUnKgzTnHe invoke [1]".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P invoke [2]".to_string(),
    //     "Program log: Instruction: Buy".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]".to_string(),
    //     "Program log: Instruction: Transfer".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 68608 compute units".to_string(),
    //     "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [3]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [3]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [3]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program data: vdt/007mYe4Yc5eD18zpXhNbbMHriEVHY/zUkvCF0etFVm+fXrpvmGRW0hAAAAAAAKByThgJAAABqE6XyyVIGrebKgfktIVA64kBgM+V+DmkDrglBiLQUEBmpmBoAAAAAGQC9gwHAAAAAHBl+crGAwBkVtIQAAAAAADYUq05yAIASsL40N1cvJfjKJwZfLUGKlTz2Va5zm5RFfllZ6pcs+ZfAAAAAAAAAATpKAAAAAAAqE6XyyVIGrebKgfktIVA64kBgM+V+DmkDrglBiLQUEAFAAAAAAAAADcnAgAAAAAA".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P invoke [3]".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P consumed 2006 of 52454 compute units".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P success".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P consumed 40002 of 89614 compute units".to_string(),
    //     "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P success".to_string(),
    //     "Program 11111111111111111111111111111111 invoke [2]".to_string(),
    //     "Program 11111111111111111111111111111111 success".to_string(),
    //     "Program FAdo9NCw1ssek6Z6yeWzWjhLVsr8uiCwcWNUnKgzTnHe consumed 81250 of 126721 compute units".to_string(),
    //     "Program FAdo9NCw1ssek6Z6yeWzWjhLVsr8uiCwcWNUnKgzTnHe success".to_string(),
    // ];

    // if let Some(create_event) = CreateEvent::parse_logs::<CreateEvent>(&logs) {
    //     info!("detect creating token: {:#?}", create_event);
    // } else {
    //     info!("没有解析到 CreateEvent，可能是 discriminator 不匹配或解码失败");
    // };
}
