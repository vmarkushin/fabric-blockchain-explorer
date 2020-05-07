use crate::channel::{BlockInfo, ChannelInfo, TransactionInfo};
use crate::proto_gen::chaincode::ChaincodeInput;
use crate::proto_gen::common::{
    Block, BlockMetadataIndex, ChannelHeader, Envelope, Header, HeaderType, Payload,
    SignatureHeader,
};
use crate::proto_gen::identities::SerializedIdentity;
use crate::proto_gen::proposal::ChaincodeProposalPayload;
use crate::proto_gen::transaction::{ChaincodeActionPayload, Transaction, TxValidationCode};
use protobuf::{parse_from_bytes, Message, ProtobufEnum};
use std::collections::HashMap;
use std::fs;
use std::process::{Command, Stdio};

/// Provides a convenient interface for fetching information from a node.
///
/// At now, this can:
/// - fetch information about the channel
/// - fetch block information (by height)
///
/// Note: it uses the fabric's `peer` utility for retrieving the information.
pub struct PeerCmd {
    /// Fabric project path
    fabric_path: String,
    /// Cached blocks
    loaded_blocks: HashMap<u64, BlockInfo>,
    /// Name of the channel to work with
    channel_name: String,
}

impl PeerCmd {
    /// Sets up environment for the utility and creates a new instance of `PeerCmd`.
    ///
    /// Note: `FABRIC_PROJ_PATH` var should be set.
    pub fn new(channel_name: &str) -> Self {
        let fabric_path =
            std::env::var("FABRIC_PROJ_PATH").expect("please set FABRIC_PROJ_PATH var");
        std::env::set_var("FABRIC_CFG_PATH", format!("{}/../config/", fabric_path));
        std::env::set_var("CORE_PEER_TLS_ENABLED", "true");
        std::env::set_var("CORE_PEER_LOCALMSPID", "Org1MSP");
        std::env::set_var("CORE_PEER_TLS_ROOTCERT_FILE", format!("{}/organizations/peerOrganizations/org1.example.com/peers/peer0.org1.example.com/tls/ca.crt", fabric_path));
        std::env::set_var(
            "CORE_PEER_MSPCONFIGPATH",
            format!(
                "{}/organizations/peerOrganizations/org1.example.com/users/Admin@org1.example.com/msp",
                fabric_path
            ),
        );
        std::env::set_var("CORE_PEER_ADDRESS", "localhost:7051");

        Self {
            fabric_path,
            loaded_blocks: Default::default(),
            channel_name: channel_name.to_owned(),
        }
    }

    /// A wrapper around protobuf's `parse_from_bytes` function.
    fn parse_pb<T: Message>(bytes: &[u8]) -> Option<T> {
        parse_from_bytes(bytes)
            .map_err(|e| eprintln!("couldn't parse `{}`: {}", std::any::type_name::<T>(), e))
            .ok()
    }

    /// Creates a `Command` with preinitialized properties. This command will call `peer` util.
    fn cmd(&self) -> Command {
        let mut cmd = Command::new(format!("{}/../bin/peer", &self.fabric_path));
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        cmd
    }

    /// Fetches information about the channel.
    pub fn get_channel_info(&self) -> Option<ChannelInfo> {
        let peer_cmd = self
            .cmd()
            .arg("channel")
            .arg("getinfo")
            .arg("-c")
            .arg(&self.channel_name)
            .spawn()
            .map_err(|e| eprintln!("failed to execute command 'peer' {}", e))
            .ok()?;

        let output = peer_cmd.wait_with_output().ok()?;
        if !output.status.success() {
            eprintln!("'peer' error code: {}", output.status);
            return None;
        }

        let out = String::from_utf8(output.stdout).ok()?;
        // the returned string doesn't begin with JSON-data, so we should find it ourselves
        out.find("{").and_then(|i| {
            serde_json::from_str(&out[i..])
                .map_err(|e| eprintln!("{}", e))
                .ok()
        })
    }

    /// Tries to load block info from cache, otherwise fetches it from the node.
    pub fn fetch_and_get_block(&mut self, height: u64) -> Option<BlockInfo> {
        if let Some(block_info) = self.loaded_blocks.get(&height) {
            return Some(block_info.clone());
        }

        let our_path = "peer_cmd_out";
        let mut peer_cmd = self
            .cmd()
            .arg("channel")
            .arg("fetch")
            .arg("-c")
            .arg(&self.channel_name)
            .arg(format!("{}", height))
            .arg(our_path)
            .spawn()
            .map_err(|e| eprintln!("failed to execute command 'peer' {}", e))
            .ok()?;
        let _ = peer_cmd.wait().ok()?;

        let bytes = fs::read(our_path)
            .map_err(|e| eprintln!("couldn't open the file: {}", e))
            .ok()?;
        let block: Block = Self::parse_pb(&bytes)?;
        let block_header = block.get_header();
        let height = block_header.get_number();
        let hash = hex::encode(block_header.get_data_hash());

        let mut transactions = Vec::new();
        let mut enveloper_ind = 0;
        for enveloper_data in block.get_data().get_data() {
            // basic information about envelope
            let envelope: Envelope = Self::parse_pb(enveloper_data)?;
            let envelope_payload: Payload = Self::parse_pb(envelope.get_payload())?;
            let envelope_header = envelope_payload.get_header();
            let channel_header: ChannelHeader =
                Self::parse_pb(envelope_header.get_channel_header())?;

            // collect transaction info
            let sig: SignatureHeader = Self::parse_pb(envelope_header.get_signature_header())?;
            let identity: SerializedIdentity = Self::parse_pb(sig.get_creator())?;
            let mspid = identity.mspid.clone();
            let tx_id = channel_header.get_tx_id().to_owned();
            let ts = channel_header.get_timestamp();
            let timestamp = chrono::NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32);
            let valid_code: u8 = block
                .get_metadata()
                .get_metadata()
                .get(BlockMetadataIndex::TRANSACTIONS_FILTER.value() as usize)
                .unwrap()[enveloper_ind];
            let valid = valid_code as i32 == TxValidationCode::VALID.value();

            transactions.push(TransactionInfo {
                valid,
                tx_id,
                timestamp,
                mspid,
            });

            let header: &Header = envelope_header;
            let ch_header: ChannelHeader = Self::parse_pb(header.get_channel_header())?;
            let _ch_name = &ch_header.channel_id;
            let header_type =
                HeaderType::from_i32(ch_header.field_type).unwrap_or(HeaderType::MESSAGE);

            if let HeaderType::ENDORSER_TRANSACTION = header_type {
                let tx: Transaction = Self::parse_pb(&envelope.payload)?;
                let actions = tx.get_actions();
                for act in actions {
                    // TODO: this code isn't working. Need to figure out why
                    if let Some(act_payload) =
                        Self::parse_pb::<ChaincodeActionPayload>(act.get_payload())
                    {
                        let chaincode_endorsed_act = act_payload.get_action();
                        let _endorsements = chaincode_endorsed_act.get_endorsements();
                        let chaincode_proposal_payload: ChaincodeProposalPayload =
                            Self::parse_pb(act_payload.get_chaincode_proposal_payload())?;
                        let _chaincode_input: ChaincodeInput =
                            Self::parse_pb(chaincode_proposal_payload.get_input())?;
                    }
                }
            }
            enveloper_ind += 1;
        }

        let block_info = BlockInfo {
            height,
            transactions,
            hash,
        };
        self.loaded_blocks.insert(height, block_info.clone());
        Some(block_info)
    }
}
