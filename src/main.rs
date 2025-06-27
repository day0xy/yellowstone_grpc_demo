mod client;
mod filters;
mod handles;
mod types;
mod utils;

use dotenvy::dotenv;
use futures::{sink::SinkExt, stream::StreamExt};
use log::info;
use std::env;
use tokio::time::{Duration, interval};

use yellowstone_grpc_proto::prelude::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterSlots, SubscribeRequestPing,
    SubscribeUpdatePong, SubscribeUpdateSlot, subscribe_update::UpdateOneof,
};

use client::connection::GrpcClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // 配置日志格式，使用本地时区（最快方案）
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
    let grpc_client = GrpcClient::new(endpoint, None);
    let mut client = grpc_client.build_client().await?;
    let (mut subscribe_tx, mut stream) = client.subscribe().await?;

    let subscribe_request_filter_slot = SubscribeRequest {
        slots: maplit::hashmap! {
            "".to_owned() => SubscribeRequestFilterSlots{
                filter_by_commitment:Some(true),
                interslot_updates:Some(false),
            }

        },
        commitment: Some(CommitmentLevel::Processed as i32),
        ..Default::default()
    };

    futures::try_join!(
        async move {
            subscribe_tx.send(subscribe_request_filter_slot).await?;

            let mut timer = interval(Duration::from_secs(3));
            let mut id = 0;
            loop {
                timer.tick().await;
                id += 1;
                subscribe_tx
                    .send(SubscribeRequest {
                        ping: Some(SubscribeRequestPing { id }),
                        ..Default::default()
                    })
                    .await?;
            }

            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        },
        async move {
            while let Some(message) = stream.next().await {
                match message?.update_oneof.expect("valid message") {
                    UpdateOneof::Slot(SubscribeUpdateSlot { slot, .. }) => {
                        info!("slog received {slot}")
                    }
                    UpdateOneof::Ping(_msg) => {
                        info!("ping received!")
                    }
                    UpdateOneof::Pong(SubscribeUpdatePong { id }) => {
                        info!("pong received id${id}")
                    }
                    msg => anyhow::bail!("receive unexpected message: {msg:?}"),
                }
            }

            Ok::<(), anyhow::Error>(())
        }
    )?;

    Ok(())
}
