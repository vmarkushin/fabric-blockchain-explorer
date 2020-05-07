use chrono::NaiveDateTime;

/// Contains information about a transaction.
#[derive(Clone)]
pub struct TransactionInfo {
    pub(crate) valid: bool,
    pub(crate) tx_id: String,
    pub(crate) timestamp: NaiveDateTime,
    pub(crate) mspid: String,
}

/// Contains information about a channel.
#[derive(Deserialize)]
pub struct ChannelInfo {
    pub(crate) height: u64,
    #[serde(rename = "currentBlockHash")]
    #[serde(default)]
    pub(crate) _current_block_hash: String,
    #[serde(rename = "previousBlockHash")]
    #[serde(default)]
    pub(crate) _previous_block_hash: String,
}

/// Contains information about a block.
#[derive(Clone)]
pub struct BlockInfo {
    pub(crate) height: u64,
    pub(crate) hash: String,
    pub(crate) transactions: Vec<TransactionInfo>,
}
