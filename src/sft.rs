use crate::operation::{MintRecord, SplitRecord};
use anyhow::Result;
use ciborium::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    pub content_type: String,
    pub body: Vec<u8>,
}

impl Content {
    pub fn new(content_type: String, body: Vec<u8>) -> Self {
        Self { content_type, body }
    }
    pub fn text(body: String) -> Self {
        Self::new("text/plain".to_string(), body.as_bytes().to_vec())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SFT {
    pub tick: String,
    pub amount: u64,
    pub attributes: Value,
    pub content: Option<Content>,
}

impl SFT {
    pub fn new(tick: String, amount: u64, attributes: Value, content: Option<Content>) -> Self {
        Self {
            tick,
            amount,
            attributes,
            content,
        }
    }

    pub fn split(&mut self, amount: u64) -> Result<SFT> {
        if amount > self.amount {
            return Err(anyhow::anyhow!(
                "Split amount is greater than the SFT amount"
            ));
        }
        self.amount -= amount;
        Ok(SFT::new(
            self.tick.clone(),
            amount,
            self.attributes.clone(),
            self.content.clone(),
        ))
    }

    pub fn merge(&mut self, sft: SFT) -> Result<()> {
        if self.tick != sft.tick {
            return Err(anyhow::anyhow!("SFTs have different ticks"));
        }
        if self.attributes != sft.attributes {
            return Err(anyhow::anyhow!("SFTs have different attributes"));
        }
        if self.content != sft.content {
            return Err(anyhow::anyhow!("SFTs have different content"));
        }
        self.amount += sft.amount;
        Ok(())
    }

    pub fn to_mint_record(&self) -> MintRecord {
        MintRecord {
            tick: self.tick.clone(),
            sft: self.clone(),
        }
    }

    pub fn to_split_record(&self) -> SplitRecord {
        SplitRecord {
            tick: self.tick.clone(),
            sft: self.clone(),
        }
    }
}
