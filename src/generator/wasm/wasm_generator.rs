use crate::generator::{Generator, InscribeGenerateOutput, InscribeSeed};
use bitcoin::Address;
use ciborium::Value;
use serde_json;
use serde_json::{Number, Value as JSONValue};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use wasmer::Value::I32;
use wasmer::*;

use tracing::{error, debug};

#[derive(Clone)]
pub struct WASMGenerator {
    bytecode: Vec<u8>,
}

impl WASMGenerator {
    pub fn new(bytecode: Vec<u8>) -> Self {
        Self { bytecode }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct Env {
    memory: Option<Arc<Mutex<Memory>>>,
}

fn fd_write(env: FunctionEnvMut<Env>, _fd: i32, mut iov: i32, iovcnt: i32, pnum: i32) -> i32 {
    let mut written_bytes = 0;

    if let Some(memory_obj) = env.data().memory.clone() {
        debug!("fd_write: fd:{}, iov:{}, iovcnt:{}, pnum:{}", _fd, iov, iovcnt, pnum);

        let memory = memory_obj.lock().expect("getting memory mutex failed");
        let store_ref = env.as_store_ref();
        let memory_view = memory.view(&store_ref);

        let mut temp_buffer: [u8; 4] = [0; 4];

        for _ in 0..iovcnt {
            let ptr_index = iov;
            let len_index = iov + 4;

            memory_view
                .read(ptr_index as u64, temp_buffer.as_mut_slice())
                .expect("read data from memory view failed");
            let _ptr = u32::from_le_bytes(temp_buffer);

            memory_view
                .read(len_index as u64, temp_buffer.as_mut_slice())
                .expect("read data from memory view failed");
            let len = u32::from_le_bytes(temp_buffer);

            debug!("fd_write: _ptr:{}, len:{}", _ptr, len);
            
            let mut buffer = vec![0u8; len as usize];
            memory_view
                .read(_ptr as u64, &mut buffer)
                .expect("read buffer from memory failed");
            
            match _fd {
                // stdout
                1 => {
                    use std::io::{self, Write};
                    let stdout = io::stdout();
                    let mut handle = stdout.lock();
                    handle.write_all(&buffer).expect("write to stdout failed");
                },
                // stderr
                2 => {
                    use std::io::{self, Write};
                    let stderr = io::stderr();
                    let mut handle = stderr.lock();
                    handle.write_all(&buffer).expect("write to stderr failed");
                },
                // Handle other file descriptors...
                _ => unimplemented!(),
            }

            iov += 8;
            written_bytes += len as i32;
        }

        let ret_index = pnum;
        let ret_index_bytes: [u8; 4] = written_bytes.to_le_bytes();
        memory_view
            .write(ret_index as u64, ret_index_bytes.as_slice())
            .expect("write data to memory failed");
    }

    written_bytes
}

fn convert_i32_pair_to_i53_checked(lo: i32, hi: i32) -> i32 {
    let p0 = if lo > 0 { 1 } else { 0 };
    let p1 = (hi + 0x200000) >> 0 < (0x400001 - p0);
    if p1 {
        let (e0, _) = (hi as u32).overflowing_add_signed(429496729);
        let (e1, _) = (lo >> 0).overflowing_add_unsigned(e0);
        e1
    } else {
        0
    }
}

fn fd_seek(
    _env: FunctionEnvMut<Env>,
    _fd: i32,
    offset_low: i64,
    offset_high: i32,
    _whence: i32,
) -> i32 {
    let _offset = convert_i32_pair_to_i53_checked(offset_low as i32, offset_high);
    return 70;
}

fn fd_close(_env: FunctionEnvMut<Env>, _fd: i32) -> i32 {
    0
}

fn proc_exit(_env: FunctionEnvMut<Env>, code: i32) {
    error!("program exit with {:}", code)
}

fn put_data_on_stack(memory: &mut Arc<Mutex<Memory>>, stack_alloc_func: &Function, store: &mut Store, data: &[u8]) -> i32 {
    let data_len = data.len() as i32;
    let result = stack_alloc_func
        .call(store, vec![I32(data_len + 1)].as_slice())
        .expect("call stackAlloc failed");
    let return_value = result
        .deref()
        .get(0)
        .expect("the stackAlloc func does not have return value");
    let offset = return_value
        .i32()
        .expect("the return value of stackAlloc is not i32");

    let bindings = memory.lock().expect("getting memory mutex failed");
    let memory_view = bindings.view(store);
    memory_view
        .write(offset as u64, data)
        .expect("write memory failed");

    offset
}

fn get_data_from_heap(memory: &mut Arc<Mutex<Memory>>, store: &Store, ptr_offset: i32) -> Vec<u8> {
    let bindings = memory.lock().expect("getting memory mutex failed");
    let memory_view = bindings.view(store);
    let mut length_bytes: [u8; 4] = [0; 4];
    memory_view
        .read(ptr_offset as u64, length_bytes.as_mut_slice())
        .expect("read length_bytes failed");
    let length = u32::from_be_bytes(length_bytes);
    let mut data = vec![0; length as usize];
    memory_view
        .read((ptr_offset + 4) as u64, &mut data)
        .expect("read uninit failed");
    data
}

fn create_wasm_instance(bytecode: &Vec<u8>) -> (Instance, Store) {
    let mut store = Store::default();
    let module = Module::new(&store, bytecode).unwrap();

    let env = FunctionEnv::new(
        &mut store,
        Env {
            memory: None,
        },
    );

    let import_object = imports! {
        "wasi_snapshot_preview1" => {
            "fd_write" => Function::new_typed_with_env(&mut store, &env, fd_write),
            "fd_seek" => Function::new_typed_with_env(&mut store, &env, fd_seek),
            "fd_close" => Function::new_typed_with_env(&mut store, &env, fd_close),
            "proc_exit" => Function::new_typed_with_env(&mut store, &env, proc_exit),
        }
    };

    let instance = Instance::new(&mut store, &module, &import_object).unwrap();

    let memory = instance.exports.get_memory("memory").unwrap();
    env.as_mut(&mut store).memory = Some(Arc::new(Mutex::new(memory.clone())));

    return (instance, store);
}

impl Generator for WASMGenerator {
    fn inscribe_generate(
        &self,
        deploy_args: Vec<String>,
        seed: &InscribeSeed,
        _recipient: Address,
        user_input: Option<String>,
    ) -> InscribeGenerateOutput {
        let (instance, mut store) = create_wasm_instance(&self.bytecode);
        let stack_alloc_func = instance.exports.get_function("stackAlloc").unwrap();
        let inscribe_generate = instance.exports.get_function("inscribe_generate").unwrap();

        let memory_obj = instance.exports.get_memory("memory").unwrap();
        let mut memory = Arc::new(Mutex::new(memory_obj.clone()));

        let mut mint_args_json: Vec<JSONValue> = vec![];
        for arg in deploy_args.iter() {
            let arg_json: JSONValue =
                serde_json::from_str(arg.as_str()).expect("serder_json unmarshal failed");
            mint_args_json.push(arg_json);
        }

        let mint_args_array = JSONValue::Array(mint_args_json);
        let mut cbor_buffer = Vec::new();
        ciborium::into_writer(&mint_args_array, &mut cbor_buffer).expect("ciborium marshal failed");

        let mut attrs_buffer_vec = Vec::new();
        for byte in cbor_buffer.iter() {
            attrs_buffer_vec.push(serde_json::Value::Number(Number::from(byte.clone())));
        }

        let mut buffer_map = serde_json::Map::new();

        buffer_map.insert(
            "attrs".to_string(),
            serde_json::Value::Array(attrs_buffer_vec),
        );

        let seed = seed.seed().to_string();
        buffer_map.insert("seed".to_string(), serde_json::Value::String(seed));

        if user_input.is_some() {
            buffer_map.insert(
                "user_input".to_string(),
                serde_json::Value::String(user_input.unwrap()),
            );
        }

        let top_buffer_map = JSONValue::Object(buffer_map);
        let mut top_buffer = Vec::new();
        ciborium::into_writer(&top_buffer_map, &mut top_buffer).expect("ciborium marshal failed");

        let mut buffer_final = Vec::new();
        buffer_final.append(&mut (top_buffer.len() as u32).to_be_bytes().to_vec());
        buffer_final.append(&mut top_buffer);

        let buffer_final_ptr =
            put_data_on_stack(&mut memory, stack_alloc_func, &mut store, buffer_final.as_slice());

        let func_args = 
        vec![I32(buffer_final_ptr)];

        let calling_result = inscribe_generate
            .call(&mut store, func_args.as_slice())
            .expect("call inscribe_generate failed");

        let return_value = calling_result.deref().get(0).unwrap();
        let offset = return_value.i32().unwrap();

        let data = get_data_from_heap(&mut memory, &store, offset);

        let return_value: Value =
            ciborium::from_reader(data.as_slice()).expect("ciborium::from_reader failed");

        let mut inscribe_generate_output = InscribeGenerateOutput::default();

        for (k, v) in return_value
            .as_map()
            .expect("the return value from inscribe_generate is incorrect")
        {
            if k.as_text().is_some() {
                let key = k.as_text().unwrap().to_string();
                if key == "amount" {
                    let value = u128::try_from(v.as_integer().unwrap()).unwrap();
                    inscribe_generate_output.amount = value as u64;
                }
                if key == "attributes" {
                    inscribe_generate_output.attributes = Some(v.clone())
                }
            }
        }

        inscribe_generate_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::sha256d;
    use bitcoin::{Address, Network};
    use bitcoin::{BlockHash, Txid};
    use std::fs::read;
    use std::str::FromStr;
    use env_logger;
     
    #[test]
    fn test_inscribe_generate_normal() {
        env_logger::init();

        // Read WASM binary from file
        let bytecode = read("./generator/cpp/generator.wasm").expect("failed to read WASM file");
        let generator = WASMGenerator::new(bytecode);

        let deploy_args = vec![r#"{"height":{"type":"range","data":{"min":1,"max":1000}}}"#.to_string()];

        // Block hash
        let block_hash_hex = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";
        let block_hash_inner = sha256d::Hash::from_str(&block_hash_hex).unwrap();
        let block_hash = BlockHash::from(block_hash_inner);

        // Txid
        let txid_hex = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b";
        let txid_inner = sha256d::Hash::from_str(&txid_hex).unwrap();
        let txid = Txid::from(txid_inner);

        let seed = InscribeSeed {
            block_hash,
            utxo: bitcoin::OutPoint::new(txid, 0),
        };

        // Recipient
        let recipient: Address = Address::from_str("32iVBEu4dxkUQk9dJbZUiBiQdmypcEyJRf").unwrap()
            .require_network(Network::Bitcoin).unwrap();

        // User input
        let user_input = Some("test user input".to_string());

        let output = generator.inscribe_generate(deploy_args, &seed, recipient, user_input);

        // Add assertions for output
        assert_eq!(output.amount, 1000);
        assert!(output.attributes.is_some());

        // Check if attributes contain expected key-value pairs
        let attributes = output.attributes.unwrap();
        assert!(attributes.is_map());
        let map = attributes.as_map().unwrap();

        let height_entry = map.iter().find(|(key, _)| key.as_text() == Some("height"));
        assert!(height_entry.is_some());

        let (_, height_value) = height_entry.unwrap();
        assert!(height_value.is_integer());
        let height = height_value.as_integer().unwrap();
        assert!(height.try_into().unwrap_or(0) >= 1);
        assert!(height.try_into().unwrap_or(i128::MAX) <= 1000);
    }
}