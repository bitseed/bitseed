#![no_std]
#![no_main]

use alloc::vec::Vec;
use minicbor::{Decoder, Encoder, Decode, Encode};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// 假设你有一个输入和输出的结构体，你需要根据你的应用程序适当定义它们
#[derive(Encode, Decode)]
struct InputData {
    #[n(0)] left: usize,
    #[n(1)] right: usize,
}

#[derive(Encode, Decode)]
struct OutputData {
    #[n(0)] sum: usize,
}

fn process_input_data(input_data: InputData) -> OutputData {
    let sum = input_data.left + input_data.right;

    OutputData { sum }
}

// 由于我们在 no_std 环境中，我们不能使用 wasm_bindgen，但是我们可以定义一个类似的接口
// 如果你需要在 wasm 环境中使用它，你需要确保你的构建环境支持 alloc
pub fn inscribe_generate(input: Vec<u8>) -> Vec<u8> {
    // 创建一个解码器
    let mut dec = Decoder::new(&input);
    // 尝试从 CBOR 编码的输入解码
    let input_data: InputData = match dec.decode() {
        Ok(data) => data,
        Err(e) => {
            // 处理解码错误，例如返回错误编码的 CBOR 或 panic
            panic!("Failed to decode input: {:?}", e);
        }
    };

    // 执行你的业务逻辑...
    let output_data = process_input_data(input_data);

    // 创建一个编码器
    let mut enc = Encoder::new(Vec::new());
    // 将输出数据编码为 CBOR
    enc.encode(&output_data).expect("Failed to encode output");
    // 获取编码后的数据
    enc.into_inner()
}
