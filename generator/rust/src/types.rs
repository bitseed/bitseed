use minicbor::{Decode, Encode};
use super::constants::{MAX_STRING_LEN, MAX_DEPLOY_ARGS, MAX_CONTENT_SIZE };

pub struct InputData<'a> {
    pub deploy_args: [[u8; MAX_STRING_LEN]; MAX_DEPLOY_ARGS],
    pub seed: &'a str,
    pub user_input: &'a str,
}

impl<'a, C> Decode<'a, C> for InputData<'a> {
    fn decode(d: &mut minicbor::Decoder<'a>, _ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let mut deploy_args = [[0; MAX_STRING_LEN]; MAX_DEPLOY_ARGS];
        let mut seed = "test";
        let mut user_input = "test";

        Ok(InputData {
            deploy_args,
            seed,
            user_input,
        })
    }
}

pub struct Content<'a> {
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

pub struct OutputData<'a> {
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

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_ENCODED_INPUT_SIZE: usize = 1024;

    #[test]
    fn test_input_data_decode() {
        let input = [b'a', b'r', b'g', b'1', 0, 0, 0, 0, 0, 0];

        let decoded_input = minicbor::decode::<InputData>(&input).unwrap();

        assert_eq!(decoded_input.deploy_args.len(), 1);
        assert_eq!(decoded_input.seed, "test");
        assert_eq!(decoded_input.user_input, "test");
    }
}
