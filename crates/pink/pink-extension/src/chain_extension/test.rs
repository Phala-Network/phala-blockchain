use scale::{Decode, Encode};

pub struct MockExtension<F, I, O, const FID: u32> {
    call: F,
    _p: std::marker::PhantomData<(I, O)>,
}

impl<F, In, Out, const FID: u32> ink::env::test::ChainExtension for MockExtension<F, In, Out, FID>
where
    In: Decode,
    Out: Encode,
    F: FnMut(In) -> Out,
{
    fn func_id(&self) -> u32 {
        FID
    }

    fn call(&mut self, input: &[u8], output: &mut Vec<u8>) -> u32 {
        let input: Vec<u8> = Decode::decode(&mut &input[..]).expect("mock decode input failed");
        let input = In::decode(&mut &input[..]).expect("mock decode input failed");
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
