mod client;
mod filters;
mod handles;
mod types;
mod utils;

use chrono::{Local, Utc};
use dotenvy::dotenv;
use futures::{sink::SinkExt, stream::StreamExt};
use log::info;
use serde_json::{Value, json};
use std::io::Write;
use std::{collections::HashMap, env};
use tokio::time::{Duration, interval};

use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_proto::prelude::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestPing, SubscribeUpdatePong,
    SubscribeUpdateSlot, subscribe_update::UpdateOneof,
};

use client::connection::GrpcClient;

use filters::filter_account::new_filter_accounts;
use filters::filter_transaction::new_filter_transactions;
use utils::format::create_pretty_transaction;

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
    let account_include = vec!["3Z19SwGej4xwKh9eiHyx3eVWHjBDEgGHeqrKtmhNcxsv".to_string()];
    let mut client = grpc.build_client().await?;

    let request = new_filter_transactions(account_include, None, None);

    let (mut subscribe_tx, mut stream) = client.subscribe_with_request(Some(request)).await?;

    while let Some(message) = stream.next().await {
        match message?.update_oneof.expect("invalid message") {
            UpdateOneof::Account(subscribe_account) => {
                if let Some(account) = subscribe_account.account {
                    info!("account lamports: {}", account.lamports);
                    let account_pubkey = Pubkey::try_from(account.pubkey.as_slice())?;
                    info!("account_pubkey: {:#?}", account_pubkey);
                }
            }
            UpdateOneof::Transaction(msg) => {
                let tx = msg
                    .transaction
                    .ok_or(anyhow::anyhow!("no transaction in the message"))?;
                let mut value = create_pretty_transaction(tx)?;
                value["slot"] = json!(msg.slot);
                info!(
                    "Receive transaction: {}",
                    serde_json::to_string(&value).expect("json serialization failed")
                );
            }
            UpdateOneof::Slot(SubscribeUpdateSlot { slot, .. }) => {
                info!("slog received {slot}")
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
                info!("pong received id${id}")
            }
            msg => anyhow::bail!("receive unexpected message: {msg:?}"),
        }
    }

    Ok::<(), anyhow::Error>(())
}
