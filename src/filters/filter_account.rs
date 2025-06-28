use std::collections::HashMap;
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts,
};

pub fn new_filter_accounts(
    account: Option<Vec<String>>,
    owner: Option<Vec<String>>,
) -> SubscribeRequest {
    SubscribeRequest {
        accounts: HashMap::from([(
            "client".to_string(),
            SubscribeRequestFilterAccounts {
                account: account.unwrap_or_default(),
                owner: owner.unwrap_or_default(),
                ..Default::default()
            },
        )]),
        commitment: Some(CommitmentLevel::Processed.into()),
        ..Default::default()
    }
}

pub fn monitor_wallet_3z() -> SubscribeRequest {
    new_filter_accounts(
        Some(vec![
            "3Z19SwGej4xwKh9eiHyx3eVWHjBDEgGHeqrKtmhNcxsv".to_string(),
        ]),
        None,
    )
}
