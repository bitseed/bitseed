#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

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

use constants::MAX_CONTENT_SIZE;
use types::{ InputData, OutputData, Content };

static mut OUTPUT_BUFFER: [u8; 16] = [0; 16];

#[no_mangle]
pub extern "C" fn inscribe_generate(input: &[u8]) -> &'static [u8] {
    #[cfg(feature = "debug")]
    console_log("inscribe_generate_start");

    let input_data = minicbor::decode::<InputData>(input).unwrap();

    #[cfg(feature = "debug")]
    console_log("input_data seed:");

    #[cfg(feature = "debug")]
    console_log(input_data.seed);

    let mut content = [0; MAX_CONTENT_SIZE];
    let msg = b"Hello, World!";
    content[..msg.len()].copy_from_slice(msg);

    let output_data = OutputData {
        amount: 1000,
        attributes: None,
        content: Some(Content {
            content_type: "text/plain",
            content,
            content_len: msg.len(),
        }),
    };

    unsafe {
        minicbor::encode(&output_data, OUTPUT_BUFFER.as_mut()).unwrap();
        &OUTPUT_BUFFER
    }
}
