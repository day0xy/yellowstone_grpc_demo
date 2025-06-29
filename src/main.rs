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
    let account_include = vec!["39H3DGBpHpffjTuwQDR9yv9AgbK4U4hesLdsVZ9yDDc9".to_string()];
    let mut client = grpc.build_client().await?;

    let request = new_filter_transactions(account_include, None, None);
    // let request = new_filter_accounts(Some(account_include), None);

    let logs = vec![
        "Program ComputeBudget111111111111111111111111111111 invoke [1]".to_string(),
        "Program ComputeBudget111111111111111111111111111111 success".to_string(),
        "Program EBgFGVhURVjFudGD84L1Ag59x3etohaBLtqwXRbcwrxJ invoke [1]".to_string(),
        "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P invoke [2]".to_string(),
        "Program log: Instruction: Sell".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]".to_string(),
        "Program log: Instruction: Transfer".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 74642 compute units".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
        "Program data: vdt/007mYe4Ijo23/Emj3q/3M3/5ts/I5BQWURKiXuPbWHDrJD5736aYpzoAAAAAztrQA5geAAAAH9YnmgnnOM/xnKl5duleNh2Pxr//9RkITp9tepH2fAaJnGBoAAAAAHOG/wcHAAAAMC1sXHXJAwBz2tsLAAAAADCVWRDkygIArRHmpPwpRKT6glG++BVCbhv7KMa2ZGZ3YHxq2fVmpkZfAAAAAAAAAO+ljgAAAAAAx0GmLvpa1KUwffsXPFuBntyMwhosrhNDUSUPGbyNAcEFAAAAAAAAAACCBwAAAAAA".to_string(),
        "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P invoke [3]".to_string(),
        "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P consumed 2006 of 64547 compute units".to_string(),
        "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P success".to_string(),
        "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P consumed 35483 of 97190 compute units".to_string(),
        "Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P success".to_string(),
        "Program EBgFGVhURVjFudGD84L1Ag59x3etohaBLtqwXRbcwrxJ consumed 38154 of 99850 compute units".to_string(),
        "Program EBgFGVhURVjFudGD84L1Ag59x3etohaBLtqwXRbcwrxJ success".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]".to_string(),
        "Program log: Instruction: CloseAccount".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2916 of 61696 compute units".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
        "Program 11111111111111111111111111111111 invoke [1]".to_string(),
        "Program 11111111111111111111111111111111 success".to_string(),
    ];

    // 添加调试信息
    info!("开始解析日志，总共 {} 行", logs.len());
    
    // 查找包含 "Program data:" 的日志行
    for (i, log) in logs.iter().enumerate() {
        if log.contains("Program data:") {
            info!("找到 Program data 在第 {} 行: {}", i + 1, log);
        }
    }

    if let Some(create_event) = CreateEvent::parse_logs::<CreateEvent>(&logs) {
        info!("detect creating token: {:#?}", create_event);
    } else {
        info!("没有解析到 CreateEvent，可能是 discriminator 不匹配或解码失败");
    };

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

    // let (mut subscribe_tx, mut stream) = client.subscribe_with_request(Some(request)).await?;

    // while let Some(message) = stream.next().await {
    //     match message?.update_oneof.expect("invalid message") {
    //         UpdateOneof::Account(msg) => {
    //             let account = msg
    //                 .account
    //                 .ok_or(anyhow::anyhow!("no account in the message"))?;
    //             let mut value = create_pretty_account(account)?;
    //             value["isStartup"] = json!(msg.is_startup);
    //             value["slot"] = json!(msg.slot);
    //             info!(
    //                 "Receive Account: {}",
    //                 serde_json::to_string(&value).expect("json serialization failed")
    //             );
    //         }
    //         UpdateOneof::Transaction(msg) => {
    //             let tx = msg
    //                 .transaction
    //                 .ok_or(anyhow::anyhow!("no transaction in the message"))?;
    //             let meta = tx.meta.unwrap_or_default();
    //             let logs = &meta.log_messages;
    //             println!("{:?}", logs);
    //             if let Some(create_event) = CreateEvent::parse_logs::<CreateEvent>(logs) {
    //                 info!("detect creating token: {:#?}", create_event);
    //             };

    //             // let mut value = create_pretty_transaction(tx)?;
    //             // value["slot"] = json!(msg.slot);
    //             // info!(
    //             //     "Receive transaction: {}",
    //             //     serde_json::to_string(&value).expect("json serialization failed")
    //             // );
    //         }
    //         UpdateOneof::Slot(msg) => {
    //             let status =
    //                 SlotStatus::try_from(msg.status).context("failed to decode commitment")?;
    //             let value = json!({
    //                 "slot": msg.slot,
    //                 "parent": msg.parent,
    //                 "status": status.as_str_name(),
    //                 "deadError": msg.dead_error,
    //             });
    //             info!(
    //                 "Receive Slot: {}",
    //                 serde_json::to_string(&value).expect("json serialization failed")
    //             );
    //         }
    //         UpdateOneof::Ping(_) => {
    //             let _ = subscribe_tx
    //                 .send(SubscribeRequest {
    //                     ping: Some(SubscribeRequestPing { id: 1 }),
    //                     ..Default::default()
    //                 })
    //                 .await;
    //             // info!("service is ping: {:#?}", Local::now());
    //             info!("service is ping");
    //         }
    //         UpdateOneof::Pong(SubscribeUpdatePong { id }) => {
    //             info!("pong received id{id}")
    //         }
    //         msg => anyhow::bail!("receive unexpected message: {msg:?}"),
    //     }
    // }

    Ok::<(), anyhow::Error>(())
}
