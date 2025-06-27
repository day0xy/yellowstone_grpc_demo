use anyhow::Ok;
use std::time::Duration;

use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcClient, Interceptor};

pub struct GrpcClient {
    endpoint: String,
    x_token: Option<String>,
}

impl GrpcClient {
    pub fn new(endpoint: String, x_token: Option<String>) -> Self {
        Self { endpoint, x_token }
    }

    pub async fn build_client(self) -> anyhow::Result<GeyserGrpcClient<impl Interceptor>> {
        let client = GeyserGrpcClient::build_from_shared(self.endpoint)?
            .x_token(self.x_token)?
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .connect_timeout(Duration::from_secs(10))
            .keep_alive_while_idle(true)
            .timeout(Duration::from_secs(60))
            .connect()
            .await?;

        Ok(client)
    }
}
