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
        self.metadata = self.metadata.add_string("op".to_string(), op);
        self
    }

    pub fn tick(mut self, tick: String) -> Self {
        self.metadata = self.metadata.add_string("tick".to_string(), tick);
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.metadata = self.metadata.add_u64("amount".to_string(), amount);
        self
    }

    pub fn add_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata = self.metadata.add(key, value);
        self
    }

    pub fn add_metadata_string(mut self, key: String, value: String) -> Self {
        self.metadata = self.metadata.add_string(key, value);
        self
    }

    pub fn add_metadata_u64(mut self, key: String, value: u64) -> Self {
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

    pub fn add(mut self, key: String, value: Value) -> Self {
        match &mut self.metadata {
            Value::Map(map) => {
                map.push((Value::Text(key), value));
            }
            _ => {}
        }
        self
    }

    pub fn add_string(self, key: String, value: String) -> Self {
        self.add(key, Value::Text(value))
    }

    pub fn add_u64(self, key: String, value: u64) -> Self {
        self.add(key, Value::Integer(Integer::from(value)))
    }

    pub fn add_f64(self, key: String, value: f64) -> Self {
        self.add(key, Value::Float(value))
    }

    pub fn add_bool(self, key: String, value: bool) -> Self {
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
