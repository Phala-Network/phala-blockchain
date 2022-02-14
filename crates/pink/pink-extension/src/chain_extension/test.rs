use scale::{Decode, Encode};

use super::{HttpRequest, HttpResponse, SignArgs, VerifyArgs};
pub struct MockExtension<F, I, O, const FID: u32> {
    call: F,
    _p: std::marker::PhantomData<(I, O)>,
}

impl<F, In, Out, const FID: u32> ink_env::test::ChainExtension for MockExtension<F, In, Out, FID>
where
    In: Decode,
    Out: Encode,
    F: FnMut(In) -> Out,
{
    fn func_id(&self) -> u32 {
        FID
    }

    fn call(&mut self, input: &[u8], output: &mut Vec<u8>) -> u32 {
        let input = In::decode(&mut &input[..]).expect("decode input");
        let out = (self.call)(input);
        out.encode_to(output);
        0
    }
}

impl<F, In, Out, const FID: u32> MockExtension<F, In, Out, FID>
where
    In: Decode,
    Out: Encode,
    F: FnMut(In) -> Out,
{
    pub fn new(call: F) -> Self {
        Self {
            call,
            _p: Default::default(),
        }
    }
}

use super::func_ids;

pub type MockHttpRequest<F> =
    MockExtension<F, HttpRequest, HttpResponse, { func_ids::HTTP_REQUEST }>;

pub type MockSign<'a, F> = MockExtension<F, SignArgs<'a>, Vec<u8>, { func_ids::SIGN }>;
pub type MockVerify<'a, F> = MockExtension<F, VerifyArgs<'a>, bool, { func_ids::VERIFY }>;
pub type MockDeriveSr25519Pair<F> =
    MockExtension<F, Vec<u8>, (Vec<u8>, Vec<u8>), { func_ids::DERIVE_SR25519_PAIR }>;
