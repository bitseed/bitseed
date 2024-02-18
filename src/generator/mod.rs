use bitcoin::{
    address::NetworkUnchecked, block::Header, consensus::Encodable, hashes::Hash, Address, Block,
};
use ord::{Inscription, InscriptionId};
use primitive_types::H256;
use serde::{Deserialize, Serialize};

pub(crate) mod hash;
pub(crate) mod mock;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InscribeGenerateOutput {
    pub amount: u64,
    // The inscription attributes, as a JSON object.
    pub attributes: Option<serde_json::Value>,
    pub content_type: Option<String>,
    pub content: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IndexerGenerateOutput {
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seed {
    block_header: Header,
    utxo: bitcoin::OutPoint,
}

impl Seed {
    pub fn new(block: Block, utxo: bitcoin::OutPoint) -> Self {
        Self {
            block_header: block.header,
            utxo,
        }
    }

    pub fn seed(&self) -> H256 {
        let block_hash = self.block_header.block_hash();
        let mut buffer = block_hash.as_byte_array().to_vec();
        buffer.extend_from_slice(self.utxo.txid.as_byte_array());
        buffer.extend_from_slice(&self.utxo.vout.to_le_bytes());
        hash::sha3_256_of(buffer.as_slice())
    }
}

pub trait Generator {
    fn inscribe_generate(
        deploy_args: Vec<String>,
        seed: &Seed,
        sender: Address<NetworkUnchecked>,
        user_input: String,
    ) -> InscribeGenerateOutput;
    fn inscribe_verify(
        deploy_args: Vec<String>,
        seed: &Seed,
        user_input: String,
        sender: Address<NetworkUnchecked>,
        inscribe_output: InscribeGenerateOutput,
    ) -> bool {
        let output = Self::inscribe_generate(deploy_args, seed, sender, user_input);
        output == inscribe_output
    }
    fn has_indexer_generate() -> bool {
        false
    }
    fn indexer_generate(
        deploy_args: Vec<String>,
        seed: &Seed,
        sender: Address<NetworkUnchecked>,
        user_input: String,
        inscription_id: InscriptionId,
    ) -> IndexerGenerateOutput {
        IndexerGenerateOutput::default()
    }
}
