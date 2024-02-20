use crate::{
    inscription::{BitseedInscription, InscriptionBuilder},
    sft::{Content, SFT},
};
use anyhow::{anyhow, bail, Result};
use ciborium::Value;
use ord::Inscription;

pub struct DeployRecord {
    pub tick: String,
    // The total supply of the Inscription
    pub amount: u64,
    pub generator: String,
    pub repeat: u64,
    pub deploy_args: Vec<String>,
}

impl DeployRecord {
    pub fn new_deploy_record(
        tick: String,
        amount: u64,
        generator: String,
        repeat: u64,
        deploy_args: Vec<String>,
    ) -> Self {
        Self {
            tick,
            amount,
            generator,
            repeat,
            deploy_args,
        }
    }
}

pub struct MintRecord {
    pub sft: SFT,
}

pub struct SplitRecord {
    pub sft: SFT,
}

pub struct MergeRecord {
    pub sft: SFT,
}

pub enum Operation {
    Deploy(DeployRecord),
    Mint(MintRecord),
    Split(SplitRecord),
    Merge(MergeRecord),
}

impl Operation {
    pub fn to_inscription(self) -> Inscription {
        let op = self.op();
        match self {
            Operation::Deploy(record) => {
                let attributes = ciborium::Value::Map(vec![
                    (
                        Value::Text("generator".to_string()),
                        Value::Text(record.generator.clone()),
                    ),
                    (
                        Value::Text("repeat".to_string()),
                        Value::Integer(record.repeat.into()),
                    ),
                    (
                        Value::Text("deploy_args".to_string()),
                        Value::Array(
                            record
                                .deploy_args
                                .into_iter()
                                .map(|s| Value::Text(s))
                                .collect(),
                        ),
                    ),
                ]);
                InscriptionBuilder::new()
                    .op(op.clone())
                    .tick(record.tick.clone())
                    .amount(record.amount)
                    .content(Content::text(op))
                    .attributes(attributes)
                    .finish()
            }
            Operation::Mint(record) => {
                let mut builder = InscriptionBuilder::new()
                    .op(op.clone())
                    .tick(record.sft.tick.clone())
                    .amount(record.sft.amount)
                    .content(Content::text(op.clone()));
                if let Some(attributes) = record.sft.attributes {
                    builder = builder.attributes(attributes);
                }
                builder = if let Some(content) = &record.sft.content {
                    builder.content(content.clone())
                } else {
                    builder.content(Content::text(op))
                };
                builder.finish()
            }
            Operation::Split(_record) => {
                todo!()
            }
            Operation::Merge(_record) => {
                todo!()
            }
        }
    }

    pub fn from_inscription(inscription: Inscription) -> Result<Self> {
        let bitseed_inscription = BitseedInscription::new(inscription)?;
        let op = bitseed_inscription.op()?;
        let tick = bitseed_inscription.tick()?;
        let amount = bitseed_inscription.amount()?;

        let content = bitseed_inscription.content()?;
        let content = if content.is_text() && content.as_text().unwrap() == op {
            None
        } else {
            Some(Content::new(content.content_type, content.body))
        };

        match op.as_ref() {
            "deploy" => {
                let generator = bitseed_inscription
                    .get_attribute("generator")
                    .ok_or_else(|| anyhow!("missing generator"))?
                    .as_text()
                    .ok_or_else(|| anyhow!("generator is not a string"))?
                    .to_string();
                let repeat = bitseed_inscription
                    .get_attribute("repeat")
                    .ok_or_else(|| anyhow!("missing repeat"))?
                    .as_integer()
                    .ok_or_else(|| anyhow!("repeat is not an integer"))?
                    .try_into()?;
                let deploy_args = bitseed_inscription
                    .get_attribute("deploy_args")
                    .ok_or_else(|| anyhow!("missing deploy_args"))?
                    .as_array()
                    .ok_or_else(|| anyhow!("deploy_args is not an array"))?
                    .iter()
                    .map(|v| {
                        v.as_text()
                            .map(|v| v.to_string())
                            .ok_or_else(|| anyhow!("deploy_args is not an array of strings"))
                    })
                    .collect::<Result<Vec<String>>>()?;
                Ok(Operation::Deploy(DeployRecord::new_deploy_record(
                    tick,
                    amount,
                    generator,
                    repeat,
                    deploy_args,
                )))
            }
            "mint" => {
                let attributes = bitseed_inscription.attributes();
                let sft = SFT {
                    tick,
                    amount,
                    attributes,
                    content,
                };
                Ok(Operation::Mint(MintRecord { sft }))
            }
            "split" => {
                todo!()
            }
            "merge" => {
                todo!()
            }
            _ => {
                bail!("unknown op: {}", op)
            }
        }
    }

    pub fn is_deploy(&self) -> bool {
        matches!(self, Operation::Deploy(_))
    }

    pub fn as_deploy(&self) -> Option<&DeployRecord> {
        match self {
            Operation::Deploy(record) => Some(record),
            _ => None,
        }
    }

    pub fn is_mint(&self) -> bool {
        matches!(self, Operation::Mint(_))
    }

    pub fn as_mint(&self) -> Option<&MintRecord> {
        match self {
            Operation::Mint(record) => Some(record),
            _ => None,
        }
    }

    pub fn is_split(&self) -> bool {
        matches!(self, Operation::Split(_))
    }

    pub fn as_split(&self) -> Option<&SplitRecord> {
        match self {
            Operation::Split(record) => Some(record),
            _ => None,
        }
    }

    pub fn is_merge(&self) -> bool {
        matches!(self, Operation::Merge(_))
    }

    pub fn as_merge(&self) -> Option<&MergeRecord> {
        match self {
            Operation::Merge(record) => Some(record),
            _ => None,
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
