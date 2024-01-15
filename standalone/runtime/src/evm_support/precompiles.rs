use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{
    IsPrecompileResult, Precompile, PrecompileHandle, PrecompileResult, PrecompileSet,
};
use sp_core::H160;
use sp_runtime::traits::Dispatchable;
use sp_std::marker::PhantomData;

use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};

pub struct FrontierPrecompiles<R>(PhantomData<R>);
impl<R> Default for FrontierPrecompiles<R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<R> FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
{
    pub fn used_addresses() -> [H160; 8] {
        [
            hash(1),
            hash(2),
            hash(3),
            hash(4),
            hash(5),
            hash(1024),
            hash(1025),
            hash(2048),
        ]
    }
}
impl<R> PrecompileSet for FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
    R::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <R::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<R::AccountId>>,
{
    fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
        match handle.code_address() {
            // Ethereum precompiles :
            a if a == hash(1) => Some(ECRecover::execute(handle)),
            a if a == hash(2) => Some(Sha256::execute(handle)),
            a if a == hash(3) => Some(Ripemd160::execute(handle)),
            a if a == hash(4) => Some(Identity::execute(handle)),
            a if a == hash(5) => Some(Modexp::execute(handle)),
            // Non-Frontier specific nor Ethereum precompiles :
            a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
            a if a == hash(1025) => Some(ECRecoverPublicKey::execute(handle)),
            a if a == hash(2048) => Some(substrate_call::Dispatch::<R>::execute(handle)),
            _ => None,
        }
    }

    fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
        IsPrecompileResult::Answer {
            is_precompile: Self::used_addresses().contains(&address),
            extra_cost: 0,
        }
    }
}

fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}

mod substrate_call {
    use alloc::format;
    use core::marker::PhantomData;

    use codec::{Decode, DecodeLimit};
    // Substrate
    use frame_support::{
        dispatch::{DispatchClass, GetDispatchInfo, Pays, PostDispatchInfo},
        traits::{ConstU32, Get},
    };
    use sp_runtime::traits::Dispatchable;
    // Frontier
    use fp_evm::{
        ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
        PrecompileResult,
    };
    use pallet_evm::{AddressMapping, GasWeightMapping};

    // `DecodeLimit` specifies the max depth a call can use when decoding, as unbounded depth
    // can be used to overflow the stack.
    // Default value is 8, which is the same as in XCM call decoding.
    pub struct Dispatch<T, DispatchValidator = (), DecodeLimit = ConstU32<8>> {
        _marker: PhantomData<(T, DispatchValidator, DecodeLimit)>,
    }

    impl<T, DispatchValidator, DecodeLimit> Precompile for Dispatch<T, DispatchValidator, DecodeLimit>
    where
        T: pallet_evm::Config,
        T::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
        <T::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<T::AccountId>>,
        DispatchValidator: DispatchValidateT<T::AccountId, T::RuntimeCall>,
        DecodeLimit: Get<u32>,
    {
        fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
            let input = handle.input();
            let target_gas = handle.gas_limit();
            let context = handle.context();

            let call = T::RuntimeCall::decode_with_depth_limit(DecodeLimit::get(), &mut &*input)
                .map_err(|_| PrecompileFailure::Error {
                    exit_status: ExitError::Other("decode failed".into()),
                })?;
            let info = call.get_dispatch_info();

            if let Some(gas) = target_gas {
                let valid_weight = T::GasWeightMapping::weight_to_gas(info.weight) <= gas;
                if !valid_weight {
                    return Err(PrecompileFailure::Error {
                        exit_status: ExitError::OutOfGas,
                    });
                }
            }

            let origin = T::AddressMapping::into_account_id(context.caller);

            if let Some(err) = DispatchValidator::validate_before_dispatch(&origin, &call) {
                return Err(err);
            }

            handle.record_external_cost(
                Some(info.weight.ref_time()),
                Some(info.weight.proof_size()),
                None,
            )?;
            match call.dispatch(Some(origin).into()) {
                Ok(post_info) => {
                    if post_info.pays_fee(&info) == Pays::Yes {
                        let actual_weight = post_info.actual_weight.unwrap_or(info.weight);
                        let cost = T::GasWeightMapping::weight_to_gas(actual_weight);
                        handle.refund_external_cost(
                            Some(
                                info.weight
                                    .ref_time()
                                    .saturating_sub(actual_weight.ref_time()),
                            ),
                            Some(
                                info.weight
                                    .proof_size()
                                    .saturating_sub(actual_weight.proof_size()),
                            ),
                        );
                        handle.record_cost(cost)?;
                    }

                    Ok(PrecompileOutput {
                        exit_status: ExitSucceed::Stopped,
                        output: Default::default(),
                    })
                }
                Err(e) => Err(PrecompileFailure::Error {
                    exit_status: ExitError::Other(
                        format!("dispatch execution failed: {}", <&'static str>::from(e)).into(),
                    ),
                }),
            }
        }
    }

    /// Dispatch validation trait.
    pub trait DispatchValidateT<AccountId, RuntimeCall> {
        fn validate_before_dispatch(
            origin: &AccountId,
            call: &RuntimeCall,
        ) -> Option<PrecompileFailure>;
    }

    /// The default implementation of `DispatchValidateT`.
    impl<AccountId, RuntimeCall> DispatchValidateT<AccountId, RuntimeCall> for ()
    where
        RuntimeCall: GetDispatchInfo,
    {
        fn validate_before_dispatch(
            _origin: &AccountId,
            call: &RuntimeCall,
        ) -> Option<PrecompileFailure> {
            let info = call.get_dispatch_info();
            if !(info.pays_fee == Pays::Yes && info.class == DispatchClass::Normal) {
                return Some(PrecompileFailure::Error {
                    exit_status: ExitError::Other("invalid call".into()),
                });
            }
            None
        }
    }
}
