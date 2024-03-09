#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

extern crate wee_alloc;

use minicbor::{Decode, Encode};

pub struct OutPoint {
    pub txid: String
    pub index: u64
}

pub struct InscribeSeed {
    pub block_hash: String,
    pub utxo: OutPoint,
}

struct InputData {
    pub deploy_args: Vec<String>, 
    pub seed: InscribeSeed, 
    pub recipient: String, 
    pub user_input: Option<String>
}

struct Content {
    pub content_type: String,
    pub content: Vec<u8>,
}

struct OutputData {
    pub amount: u64,
    pub attributes: Option<minicbor::Value>,
    pub content: Option<Content>,
}

impl<C> Encode<C> for InputData {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(2)?;
        e.u64(self.left as u64)?;
        e.u64(self.right as u64)?;
        Ok(())
    }
}

impl<'b, C> Decode<'b, C> for InputData {
    fn decode(
        d: &mut minicbor::Decoder<'b>,
        _ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        let array_len = d.array()?;
        if array_len != Some(2) {
            return Err(minicbor::decode::Error::message("Invalid array length"));
        }
        let left = d.u64()? as usize;
        let right = d.u64()? as usize;
        Ok(InputData { left, right })
    }
}



impl<C> Encode<C> for OutputData {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.u64(self.sum as u64)?;
        Ok(())
    }
}

impl<'b, C> Decode<'b, C> for OutputData {
    fn decode(
        d: &mut minicbor::Decoder<'b>,
        _ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        let sum = d.u64()? as usize;
        Ok(OutputData { sum })
    }
}

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static mut OUTPUT_BUFFER: [u8; 16] = [0; 16];

#[no_mangle]
pub extern "C" fn inscribe_generate(input: &[u8]) -> &'static [u8] {
    let mut decoder = minicbor::Decoder::new(input);
    let input_data = InputData::decode(&mut decoder, &mut ()).unwrap();

    let output_data = OutputData {
        sum: input_data.left + input_data.right,
    };

    unsafe {
        minicbor::encode(&output_data, OUTPUT_BUFFER.as_mut()).unwrap();
        &OUTPUT_BUFFER
    }
}
