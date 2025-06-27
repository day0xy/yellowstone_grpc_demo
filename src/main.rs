use dotenvy::dotenv;
use futures::{sink::SinkExt, stream::StreamExt};
use log::info;
use std::env;
use tokio::time::{Duration, interval};
use yellowstone_grpc_client::ClientTlsConfig;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterSlots, SubscribeRequestPing,
    SubscribeUpdatePong, SubscribeUpdateSlot, subscribe_update::UpdateOneof,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    env_logger::init();
    let endpoint = env::var("YELLOWSTONE_GRPC_URL")?;

    let mut client = GeyserGrpcClient::build_from_shared(endpoint)?
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;

    let (mut subscribe_tx, mut stream) = client.subscribe().await?;

    futures::try_join!(
        async move {
            subscribe_tx
                .send(SubscribeRequest {
                    slots: maplit::hashmap! {
                        "".to_owned() => SubscribeRequestFilterSlots{
                            filter_by_commitment:Some(true),
                            interslot_updates:Some(false),
                        }

                    },
                    commitment: Some(CommitmentLevel::Processed as i32),
                    ..Default::default()
                })
                .await?;

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
