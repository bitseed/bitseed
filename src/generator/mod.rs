use anyhow::Result;
use bitcoin::{
    address::NetworkUnchecked, block::Header, consensus::Encodable, hashes::Hash, Address, Block,
    BlockHash,
};
use ord::{Inscription, InscriptionId};
use primitive_types::H256;
use serde::{Deserialize, Serialize};

use crate::{sft::Content, wallet::Wallet};

use self::mock::random_amount_generator;

pub(crate) mod hash;
pub(crate) mod mock;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InscribeGenerateOutput {
    pub amount: u64,
    pub attributes: Option<ciborium::Value>,
    pub content: Option<Content>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IndexerGenerateOutput {
    pub attributes: Option<ciborium::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InscribeSeed {
    block_hash: BlockHash,
    utxo: bitcoin::OutPoint,
}

impl InscribeSeed {
    pub fn new(block_hash: BlockHash, utxo: bitcoin::OutPoint) -> Self {
        Self { block_hash, utxo }
    }

    pub fn seed(&self) -> H256 {
        let mut buffer = self.block_hash.as_byte_array().to_vec();
        buffer.extend_from_slice(self.utxo.txid.as_byte_array());
        buffer.extend_from_slice(&self.utxo.vout.to_le_bytes());
        hash::sha3_256_of(buffer.as_slice())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexerSeed {
    block_hash: BlockHash,
    inscription_id: InscriptionId,
}

impl IndexerSeed {
    pub fn new(block_hash: BlockHash, inscription_id: InscriptionId) -> Self {
        Self {
            block_hash,
            inscription_id,
        }
    }

    pub fn seed(&self) -> H256 {
        let mut buffer = self.block_hash.as_byte_array().to_vec();
        buffer.extend_from_slice(self.inscription_id.txid.as_byte_array());
        buffer.extend_from_slice(&self.inscription_id.index.to_le_bytes());
        hash::sha3_256_of(buffer.as_slice())
    }
}

pub const TICK: &'static str = "generator";
pub const CONTENT_TYPE: &'static str = "application/wasm";

pub trait Generator {
    fn inscribe_generate(
        &self,
        deploy_args: Vec<String>,
        seed: &InscribeSeed,
        recipient: Address,
        user_input: Option<String>,
    ) -> InscribeGenerateOutput;

    fn inscribe_verify(
        &self,
        deploy_args: Vec<String>,
        seed: &InscribeSeed,
        recipient: Address,
        user_input: Option<String>,
        inscribe_output: InscribeGenerateOutput,
    ) -> bool {
        let output = self.inscribe_generate(deploy_args, seed, recipient, user_input);
        output == inscribe_output
    }

    fn has_indexer_generate(&self) -> bool {
        false
    }

    fn indexer_generate(
        &self,
        deploy_args: Vec<String>,
        seed: &IndexerSeed,
        recipient: Address,
    ) -> IndexerGenerateOutput {
        IndexerGenerateOutput::default()
    }
}

pub struct StaticGenerator {
    pub inscribe_output: InscribeGenerateOutput,
    pub indexer_output: Option<IndexerGenerateOutput>,
}

impl StaticGenerator {
    pub fn new(
        inscribe_output: InscribeGenerateOutput,
        indexer_output: Option<IndexerGenerateOutput>,
    ) -> Self {
        Self {
            inscribe_output,
            indexer_output,
        }
    }
}

impl Generator for StaticGenerator {
    fn inscribe_generate(
        &self,
        _deploy_args: Vec<String>,
        _seed: &InscribeSeed,
        _recipient: Address,
        _user_input: Option<String>,
    ) -> InscribeGenerateOutput {
        self.inscribe_output.clone()
    }

    fn inscribe_verify(
        &self,
        _deploy_args: Vec<String>,
        _seed: &InscribeSeed,
        _recipient: Address,
        _user_input: Option<String>,
        inscribe_output: InscribeGenerateOutput,
    ) -> bool {
        self.inscribe_output == inscribe_output
    }

    fn has_indexer_generate(&self) -> bool {
        self.indexer_output.is_some()
    }

    fn indexer_generate(
        &self,
        _deploy_args: Vec<String>,
        _seed: &IndexerSeed,
        _recipient: Address,
    ) -> IndexerGenerateOutput {
        self.indexer_output.clone().unwrap()
    }
}

pub struct GeneratorLoader {
    wallet: Wallet,
}

impl GeneratorLoader {
    pub fn new(wallet: Wallet) -> Self {
        Self { wallet }
    }
    pub fn load(&self, _generator: &str) -> Result<Box<dyn Generator>> {
        //TODO load generator from Inscription
        Ok(Box::new(random_amount_generator::RandomAmountGenerator))
    }
}
