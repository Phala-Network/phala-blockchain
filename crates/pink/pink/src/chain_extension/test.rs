use scale::Decode;

pub struct MockExtensionFn<F, I, const FID: u32> {
    call: F,
    _p: std::marker::PhantomData<(I,)>,
}

impl<F, In, const FID: u32> ink::env::test::ChainExtension for MockExtensionFn<F, In, FID>
where
    In: Decode,
    F: FnMut(In) -> (u32, Vec<u8>),
{
    fn func_id(&self) -> u32 {
        FID
    }

    fn call(&mut self, input: &[u8], output: &mut Vec<u8>) -> u32 {
        let input: Vec<u8> = Decode::decode(&mut &input[..]).expect("mock decode input failed");
        let input = In::decode(&mut &input[..]).expect("mock decode input failed");
        let (status, data) = (self.call)(input);
        output.extend(&data);
        status
    }
}

impl<F, In, const FID: u32> MockExtensionFn<F, In, FID>
where
    In: Decode,
    F: FnMut(In) -> (u32, Vec<u8>),
{
    pub fn new(call: F) -> Self {
        Self {
            call,
            _p: Default::default(),
        }
    }
}
