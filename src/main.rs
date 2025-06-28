mod client;
mod filters;
mod handles;
mod types;
mod utils;

use chrono::Local;
use dotenvy::dotenv;
use futures::{sink::SinkExt, stream::StreamExt};
use log::info;
use std::{collections::HashMap, env};
use tokio::time::{Duration, interval};

use solana_sdk::{program_pack::Pack, pubkey::Pubkey, signature::Signature};
use yellowstone_grpc_proto::prelude::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterSlots, SubscribeRequestPing,
    SubscribeUpdatePong, SubscribeUpdateSlot, subscribe_update::UpdateOneof,
};

use client::connection::GrpcClient;
use filters::filter_account::monitor_wallet_3z;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            use chrono::Local;
            use std::io::Write;

            writeln!(
                buf,
                "[{} {} {}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();

    let endpoint = env::var("YELLOWSTONE_GRPC_URL")?;
    let grpc = GrpcClient::new(endpoint, None);
    let mut client = grpc.build_client().await?;
    let request = monitor_wallet_3z();

    let (mut subscribe_tx, mut stream) = client.subscribe_with_request(Some(request)).await?;

    while let Some(message) = stream.next().await {
        match message?.update_oneof.expect("invalid message") {
            UpdateOneof::Account(subscribe_account) => {
                if let Some(account) = subscribe_account.account {
                    info!("account: {:#?}", account);
                    let account_pubkey = Pubkey::try_from(account.pubkey.as_slice())?;

                    info!("account_pubkey: {:#?}", account_pubkey);

                    let owner = Pubkey::try_from(account.owner.as_slice())?;
                    info!("owner: {:#?}", owner);

                    let account_signture = Signature::try_from(account.txn_signature())?;
                    let account_info = spl_token::state::Account::unpack(&account.data)?;

                    info!("account_signture: {:#?}", account_signture);
                    info!("account_info: {:#?}", account_info);
                }
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
                info!("service is ping: {:#?}", Local::now());
            }
            UpdateOneof::Pong(SubscribeUpdatePong { id }) => {
                info!("pong received id${id}")
            }
            msg => anyhow::bail!("receive unexpected message: {msg:?}"),
        }
    }

    Ok::<(), anyhow::Error>(())
}
