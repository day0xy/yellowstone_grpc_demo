use std::collections::HashMap;

use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterTransactions,
};

pub fn new_filter_transactions(
    account_include: Vec<String>,
    account_exclude: Option<Vec<String>>,
    account_required: Option<Vec<String>>,
) -> SubscribeRequest {
    SubscribeRequest {
        transactions: HashMap::from([(
            "client".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include: account_include,
                account_exclude: account_exclude.unwrap_or_default(),
                account_required: account_required.unwrap_or_default(),
            },
        )]),

        commitment: Some(CommitmentLevel::Processed.into()),
        ..Default::default()
    }
}
