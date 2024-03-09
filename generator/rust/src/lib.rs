#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

use minicbor::{Decode, Encode};

#[cfg(feature = "debug")]
mod debug;

#[cfg(feature = "debug")]
use debug::*;

#[cfg_attr(not(any(feature = "std", feature = "debug")), panic_handler)]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

const MAX_DEPLOY_ARGS: usize = 10;
const MAX_CONTENT_SIZE: usize = 1024;
const MAX_STRING_LEN: usize = 64;

struct InputData {
    pub deploy_args: [[u8; MAX_STRING_LEN]; MAX_DEPLOY_ARGS],
    pub seed: [u8; MAX_STRING_LEN],
    pub user_input: Option<[u8; MAX_STRING_LEN]>,
}

impl<'b, C> Decode<'b, C> for InputData {
    fn decode(d: &mut minicbor::Decoder<'b>, _ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let mut deploy_args = [[0; MAX_STRING_LEN]; MAX_DEPLOY_ARGS];
        let mut seed = [0; MAX_STRING_LEN];
        let mut user_input = None;

        let mut map_decoder = d.map_iter::<&str, minicbor::data::Type>()?;

        while let Some(item) = map_decoder.next() {
            let (key, value) = item?;
            match key {
                "deploy_args" => {
                    let mut args_decoder = minicbor::Decoder::new(value.as_bytes()?);
                    for i in 0..MAX_DEPLOY_ARGS {
                        if let Some(s) = args_decoder.str()? {
                            deploy_args[i][..s.len()].copy_from_slice(s.as_bytes());
                        } else {
                            break;
                        }
                    }
                }
                "seed" => {
                    if let minicbor::data::Type::String = value {
                        let s = d.str()?;
                        seed[..s.len()].copy_from_slice(s.as_bytes());
                    }
                }
                "user_input" => {
                    if let minicbor::data::Type::String = value {
                        let s = d.str()?;
                        let mut input = [0; MAX_STRING_LEN];
                        input[..s.len()].copy_from_slice(s.as_bytes());
                        user_input = Some(input);
                    }
                }
                _ => {}
            }
        }

        Ok(InputData {
            deploy_args,
            seed,
            user_input,
        })
    }
}

struct Content<'a> {
    pub content_type: &'a str,
    pub content: [u8; MAX_CONTENT_SIZE],
    pub content_len: usize,
}

impl<'a, C> Encode<C> for Content<'a> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(2)?;
        e.str(self.content_type)?;
        e.bytes(&self.content[..self.content_len])?;
        Ok(())
    }
}

struct OutputData<'a> {
    pub amount: u64,
    pub attributes: Option<&'a [u8]>,
    pub content: Option<Content<'a>>,
}

impl<'a, C> Encode<C> for OutputData<'a> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(3)?;
        e.u64(self.amount)?;
        if let Some(attributes) = self.attributes {
            e.bytes(attributes)?;
        } else {
            e.null()?;
        }
        if let Some(content) = &self.content {
            content.encode(e, _ctx)?;
        } else {
            e.null()?;
        }
        Ok(())
    }
}

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
