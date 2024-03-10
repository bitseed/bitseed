#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(any(feature = "std", test)), no_main)]

mod constants;
mod types;

#[cfg(feature = "debug")]
mod debug;

#[cfg(feature = "debug")]
use debug::*;

#[cfg_attr(not(any(feature = "std", feature = "debug")), panic_handler)]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use constants::{MAX_CONTENT_SIZE, MAX_STRING_LEN, MAX_DEPLOY_ARGS};
use heapless::{LinearMap, String, Vec};
use types::{Content, DeployArgs, InputData, OutputData, Value};

fn hash_str_uint32(input: &str) -> u32 {
    let mut hash = 0x811c9dc5;
    let prime = 0x1000193;

    for c in input.chars() {
        let value = c as u8;
        hash ^= value as u32;
        hash = hash.wrapping_mul(prime);
    }

    hash
}

#[no_mangle]
pub extern "C" fn inscribe_generate(input: &[u8]) -> &'static [u8] {
    #[cfg(feature = "debug")]
    log!("inscribe_generate_start, input size: {}", input.len());

    let input_data = minicbor::decode::<InputData>(input).unwrap();

    #[cfg(feature = "debug")]
    log!("input_data seed: {}", input_data.seed.as_str());

    #[cfg(feature = "debug")]
    log!("input_data user_input: {}", input_data.user_input.as_str());

    let mut json_output = LinearMap::<String<MAX_STRING_LEN>, Value, MAX_DEPLOY_ARGS>::new();

    let mut seed = String::<MAX_CONTENT_SIZE>::new();
    seed.push_str(input_data.seed.as_str()).unwrap();
    seed.push_str(input_data.user_input.as_str()).unwrap();
    let hash_value = hash_str_uint32(seed.as_str());

    let deploy_args: DeployArgs =
        minicbor::decode::<DeployArgs>(input_data.deploy_args.as_slice()).unwrap();

    for arg in deploy_args.args {
        if arg.arg.type_name == "range" {
            let range_min = arg.arg.data.min;
            let range_max = arg.arg.data.max;
            let random_value = range_min + (hash_value as u64 % (range_max - range_min + 1));
            let _ = json_output.insert(arg.name.clone(), Value::UInt(random_value));
        }
    }

    let _ = json_output.insert(
        String::try_from("id").unwrap(),
        Value::String(input_data.user_input.clone()),
    );

    let output_data = OutputData {
        amount: 1,
        attributes: Some(json_output),
        content: Some(Content {
            content_type: String::new(),
            content: Vec::new(),
        }),
    };

    static mut OUTPUT_BUFFER: [u8; MAX_CONTENT_SIZE] = [0; MAX_CONTENT_SIZE];

    unsafe {
        minicbor::encode(&output_data, &mut OUTPUT_BUFFER[..]).unwrap();
        &OUTPUT_BUFFER[..minicbor::decode::<u32>(&OUTPUT_BUFFER[..4]).unwrap() as usize + 4]
    }
}
