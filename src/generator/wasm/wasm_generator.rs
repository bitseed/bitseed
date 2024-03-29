use crate::generator::{Generator, InscribeGenerateOutput, InscribeSeed};
use bitcoin::{Address, BlockHash, OutPoint};
use ciborium::Value;
use once_cell::sync::Lazy;
use serde_json;
use serde_json::{Number, Value as JSONValue};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use wasmer::Value::I32;
use wasmer::*;

#[derive(Clone)]
pub struct WASMGenerator {
    bytecode: Vec<u8>,
}

impl WASMGenerator {
    pub fn new(bytecode: Vec<u8>) -> Self {
        Self { bytecode }
    }
}

static mut GLOBAL_MEMORY: Lazy<Option<Arc<Mutex<Memory>>>> = Lazy::new(|| None);

#[allow(dead_code)]
#[derive(Clone)]
struct Env {
    memory: Option<Arc<Mutex<Memory>>>,
}

fn fd_write(env: FunctionEnvMut<Env>, _fd: i32, mut iov: i32, iovcnt: i32, pnum: i32) -> i32 {
    let memory_obj = unsafe { GLOBAL_MEMORY.clone().unwrap() };

    // let binding = env.data().memory.clone().unwrap();
    // let memory = binding.lock().unwrap();
    let memory = memory_obj.lock().expect("getting memory mutex failed");
    let store_ref = env.as_store_ref();
    let memory_view = memory.view(&store_ref);

    let mut temp_buffer: [u8; 4] = [0; 4];
    let mut number = 0;
    for _ in 0..(iovcnt - 1) {
        let ptr_index = (iov) >> 2;
        let len_index = ((iov) + (4)) >> 2;

        memory_view
            .read(ptr_index as u64, temp_buffer.as_mut_slice())
            .expect("read data from memory view failed");
        let _ptr = i32::from_be_bytes(temp_buffer);

        memory_view
            .read(len_index as u64, temp_buffer.as_mut_slice())
            .expect("read data from memory view failed");
        let len = i32::from_be_bytes(temp_buffer);

        iov += 8;
        number += len;
    }

    let ret_index = (pnum) >> 2;
    let ret_index_bytes: [u8; 4] = number.to_be_bytes();
    memory_view
        .write(ret_index as u64, ret_index_bytes.as_slice())
        .expect("write data to memory failed");
    0
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
    eprintln!("program exit with {:}", code)
}

fn put_data_on_stack(stack_alloc_func: &Function, store: &mut Store, data: &[u8]) -> i32 {
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

    let memory = unsafe { GLOBAL_MEMORY.clone().expect("global memory is none") };

    let bindings = memory.lock().expect("getting memory mutex failed");
    let memory_view = bindings.view(store);
    memory_view
        .write(offset as u64, data)
        .expect("write memory failed");

    offset
}

fn get_data_from_heap(memory: Arc<Mutex<Memory>>, store: &Store, ptr_offset: i32) -> Vec<u8> {
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

    // let ptr = memory_view.data_ptr().offset(ptr_offset as isize) as *mut c_char;
    // let c_str = CStr::from_ptr(ptr);
    // c_str.to_bytes().to_vec()
    // let rust_str = c_str.to_str().expect("Bad encoding");
    // let owned_str = rust_str.to_owned();
    // owned_str
}

fn create_wasm_instance(bytecode: &Vec<u8>) -> (Instance, Store) {
    let mut store = Store::default();
    let module = Module::new(&store, bytecode).unwrap();

    let global_memory = unsafe { GLOBAL_MEMORY.clone() };
    let env = FunctionEnv::new(
        &mut store,
        Env {
            memory: global_memory,
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
    unsafe { *GLOBAL_MEMORY = Some(Arc::new(Mutex::new(memory.clone()))) };

    return (instance, store);
}

fn join_seeds(block_hash: BlockHash, utxo: OutPoint) -> String {
    let seed_string = format!(
        "{}{}{}",
        block_hash.to_string(),
        utxo.txid.to_string(),
        utxo.vout.to_string()
    );
    seed_string
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

        let memory = unsafe { GLOBAL_MEMORY.clone().unwrap() };

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

        let seed = join_seeds(seed.block_hash, seed.utxo);
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
            put_data_on_stack(stack_alloc_func, &mut store, buffer_final.as_slice());

        let func_args = vec![I32(buffer_final_ptr)];

        let calling_result = inscribe_generate
            .call(&mut store, func_args.as_slice())
            .expect("call inscribe_generate failed");

        let return_value = calling_result.deref().get(0).unwrap();
        let offset = return_value.i32().unwrap();

        let data = get_data_from_heap(memory, &store, offset);

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
