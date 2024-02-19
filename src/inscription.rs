use crate::{sft::Content, PROTOCOL};
use ciborium::{value::Integer, Value};
use ord::Inscription;

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
        self.metadata = self.metadata.add_string("op", op);
        self
    }

    pub fn tick<S: ToString>(mut self, tick: S) -> Self {
        self.metadata = self.metadata.add_string("tick", tick.to_string());
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.metadata = self.metadata.add_u64("amount", amount);
        self
    }

    pub fn add_metadata<S: ToString>(mut self, key: S, value: Value) -> Self {
        self.metadata = self.metadata.add(key, value);
        self
    }

    pub fn add_metadata_string<S: ToString>(mut self, key: S, value: String) -> Self {
        self.metadata = self.metadata.add_string(key, value);
        self
    }

    pub fn add_metadata_u64<S: ToString>(mut self, key: S, value: u64) -> Self {
        self.metadata = self.metadata.add_u64(key, value);
        self
    }

    pub fn attributes(mut self, attributes: Value) -> Self {
        assert!(attributes.is_map());
        self.metadata = self.metadata.add("attributes".to_string(), attributes);
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
