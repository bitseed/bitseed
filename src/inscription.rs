use crate::{
    sft::Content, METADATA_AMOUNT, METADATA_ATTRIBUTES, METADATA_OP, METADATA_TICK, PROTOCOL,
};
use anyhow::{anyhow, ensure, Result};
use ciborium::{value::Integer, Value};
use ord::{Inscription, InscriptionId};

pub struct InscriptionBuilder {
    inscription: Inscription,
    metadata: MetadataBuilder,
}

impl InscriptionBuilder {
    pub fn new() -> Self {
        let mut inscription = Inscription::default();
        inscription.metaprotocol = Some(PROTOCOL.as_bytes().to_vec());
        Self {
            inscription,
            metadata: MetadataBuilder::new(),
        }
    }

    pub fn op(mut self, op: String) -> Self {
        self.metadata = self.metadata.add_string(METADATA_OP, op);
        self
    }

    pub fn tick<S: ToString>(mut self, tick: S) -> Self {
        self.metadata = self.metadata.add_string(METADATA_TICK, tick.to_string());
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.metadata = self.metadata.add_u64(METADATA_AMOUNT, amount);
        self
    }

    pub fn attributes(mut self, attributes: Value) -> Self {
        assert!(attributes.is_map());
        self.metadata = self.metadata.add(METADATA_ATTRIBUTES, attributes);
        self
    }

    pub fn content(mut self, content: Content) -> Self {
        self.inscription.content_type = Some(content.content_type.into_bytes());
        self.inscription.body = Some(content.body);
        self
    }

    pub fn finish(mut self) -> Inscription {
        self.inscription.metadata = Some(self.metadata.finish_to_bytes());
        self.inscription
    }
}

pub struct MetadataBuilder {
    metadata: Value,
}

impl MetadataBuilder {
    pub fn new() -> Self {
        Self {
            metadata: Value::Map(vec![]),
        }
    }

    pub fn add<S: ToString>(mut self, key: S, value: Value) -> Self {
        match &mut self.metadata {
            Value::Map(map) => {
                map.push((Value::Text(key.to_string()), value));
            }
            _ => {}
        }
        self
    }

    pub fn add_string<S: ToString>(self, key: S, value: String) -> Self {
        self.add(key, Value::Text(value))
    }

    pub fn add_u64<S: ToString>(self, key: S, value: u64) -> Self {
        self.add(key, Value::Integer(Integer::from(value)))
    }

    pub fn add_f64<S: ToString>(self, key: S, value: f64) -> Self {
        self.add(key, Value::Float(value))
    }

    pub fn add_bool<S: ToString>(self, key: S, value: bool) -> Self {
        self.add(key, Value::Bool(value))
    }

    pub fn finish(self) -> Value {
        self.metadata
    }

    pub fn finish_to_bytes(self) -> Vec<u8> {
        let value = self.finish();
        let mut writer = vec![];
        ciborium::into_writer(&value, &mut writer).unwrap();
        writer
    }
}

pub struct BitseedInscription {
    inscription: Inscription,
}

impl BitseedInscription {
    pub fn new(inscription: Inscription) -> Result<Self> {
        let metaprotocol = inscription
            .metaprotocol()
            .ok_or_else(|| anyhow!("metaprotocol not found"))?;
        ensure!(metaprotocol == PROTOCOL, "metaprotocol is not bitseed");
        Ok(Self { inscription })
    }

    pub fn get_metadata(&self) -> Result<Value> {
        self.inscription
            .metadata()
            .ok_or_else(|| anyhow!("metadata not found"))
    }

    pub fn get_metadata_value(&self, key: &str) -> Result<Value> {
        self.get_metadata_value_opt(key)
            .ok_or_else(|| anyhow!("key ({:?}) not found in metadata", key))
    }

    pub fn get_metadata_value_opt(&self, key: &str) -> Option<Value> {
        let metadata = self.inscription.metadata()?;
        let kvs = metadata.as_map()?;
        kvs.iter()
            .find(|(k, _)| k.is_text() && k.as_text().unwrap() == key)
            .map(|(_, v)| v.clone())
    }

    pub fn get_metadata_string(&self, key: &str) -> Result<String> {
        self.get_metadata_value(key)?
            .as_text()
            .map(|v| v.to_string())
            .ok_or_else(|| anyhow!("{} is not a string", key))
    }

    pub fn get_metadata_u64(&self, key: &str) -> Result<u64> {
        let i = self
            .get_metadata_value(key)?
            .as_integer()
            .ok_or_else(|| anyhow!("{} is not an integer", key))?;
        u64::try_from(i).map_err(|_| anyhow!("{} is not a u64", key))
    }

    pub fn op(&self) -> Result<String> {
        self.get_metadata_string(METADATA_OP)
    }

    pub fn tick(&self) -> Result<String> {
        self.get_metadata_string(METADATA_TICK)
    }

    pub fn amount(&self) -> Result<u64> {
        let amount = self
            .get_metadata_value(METADATA_AMOUNT)?
            .as_integer()
            .ok_or_else(|| anyhow!("amount is not an integer"))?;
        u64::try_from(amount).map_err(|_| anyhow!("amount is not a u64"))
    }

    pub fn attributes(&self) -> Option<Value> {
        self.get_metadata_value_opt(METADATA_ATTRIBUTES)
    }

    pub fn get_attribute(&self, key: &str) -> Option<Value> {
        self.attributes().and_then(|attributes| {
            attributes.as_map().and_then(|map| {
                map.iter()
                    .find(|(k, _)| k.is_text() && k.as_text().unwrap() == key)
                    .map(|(_, v)| v.clone())
            })
        })
    }

    pub fn content(&self) -> Option<Content> {
        let content_type = self.inscription.content_type();
        let body = self.inscription.body();
        if let (Some(content_type), Some(body)) = (content_type, body) {
            Some(Content::new(content_type.to_owned(), body.to_vec()))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct InscriptionToBurn {
    pub inscription_id: InscriptionId,
    pub message: String,
}

impl InscriptionToBurn {
    pub fn new(inscription_id: InscriptionId, message: String) -> Self {
        Self {
            inscription_id,
            message,
        }
    }
}
