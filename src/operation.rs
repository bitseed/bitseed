use crate::{
    inscription::InscriptionBuilder,
    sft::{Content, SFT},
};
use ciborium::Value;
use ord::Inscription;
use std::collections::HashMap;

pub struct DeployRecord {
    pub tick: String,
    // The total supply of the Inscription
    pub amount: u64,
    pub generator: String,
    pub repeat: u64,
    pub attributes: Value,
}

pub struct MintRecord {
    pub tick: String,
    pub sft: SFT,
}

pub struct SplitRecord {
    pub tick: String,
    pub sft: SFT,
}

pub struct MergeRecord {
    pub tick: String,
    pub sft: SFT,
}

pub enum Operation {
    Deploy(DeployRecord),
    Mint(MintRecord),
    Split(SplitRecord),
    Merge(MergeRecord),
}

impl Operation {
    pub fn inscribe(&self) -> Inscription {
        let op = self.op();
        match self {
            Operation::Deploy(record) => InscriptionBuilder::new()
                .op(op.clone())
                .tick(record.tick.clone())
                .amount(record.amount)
                .add_metadata_string("generator".to_string(), record.generator.clone())
                .add_metadata_u64("repeat".to_string(), record.repeat)
                .content(Content::text(op))
                .attributes(record.attributes.clone())
                .finish(),
            Operation::Mint(record) => {
                let mut builder = InscriptionBuilder::new()
                    .op(op.clone())
                    .tick(record.tick.clone())
                    .amount(record.sft.amount)
                    .content(Content::text(op.clone()))
                    .attributes(record.sft.attributes.clone());
                builder = if let Some(content) = &record.sft.content {
                    builder.content(content.clone())
                } else {
                    builder.content(Content::text(op))
                };
                builder.finish()
            }
            Operation::Split(record) => {
                todo!()
            }
            Operation::Merge(record) => {
                todo!()
            }
        }
    }

    pub fn op(&self) -> String {
        match self {
            Operation::Deploy(_) => "deploy".to_string(),
            Operation::Mint(_) => "mint".to_string(),
            Operation::Split(_) => "split".to_string(),
            Operation::Merge(_) => "merge".to_string(),
        }
    }
}
