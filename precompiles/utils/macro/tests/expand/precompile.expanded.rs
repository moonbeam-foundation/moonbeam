use {
    core::marker::PhantomData, precompile_utils::{EvmResult, prelude::*},
    sp_core::{H160, U256},
    frame_support::pallet_prelude::{Get, ConstU32},
};
struct BatchPrecompile<Runtime>(PhantomData<Runtime>);
type GetCallDataLimit = ConstU32<42>;
type GetArrayLimit = ConstU32<42>;
impl<Runtime> BatchPrecompile<Runtime>
where
    Runtime: Get<u32>,
{
    fn pre_check(handle: &mut impl PrecompileHandle) -> EvmResult {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("pre_check")),
        )
    }
    fn batch_some(
        handle: &mut impl PrecompileHandle,
        to: BoundedVec<Address, GetArrayLimit>,
        value: BoundedVec<U256, GetArrayLimit>,
        call_data: BoundedVec<BoundedBytes<GetCallDataLimit>, GetArrayLimit>,
        gas_limit: BoundedVec<u64, GetArrayLimit>,
    ) -> EvmResult {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("batch_some")),
        )
    }
    fn batch_some_until_failure(
        handle: &mut impl PrecompileHandle,
        to: BoundedVec<Address, GetArrayLimit>,
        value: BoundedVec<U256, GetArrayLimit>,
        call_data: BoundedVec<BoundedBytes<GetCallDataLimit>, GetArrayLimit>,
        gas_limit: BoundedVec<u64, GetArrayLimit>,
    ) -> EvmResult {
        ::core::panicking::panic_fmt(
            format_args!(
                "not yet implemented: {0}", format_args!("batch_some_until_failure")
            ),
        )
    }
    fn batch_all(
        handle: &mut impl PrecompileHandle,
        to: BoundedVec<Address, GetArrayLimit>,
        value: BoundedVec<U256, GetArrayLimit>,
        call_data: BoundedVec<BoundedBytes<GetCallDataLimit>, GetArrayLimit>,
        gas_limit: BoundedVec<u64, GetArrayLimit>,
    ) -> EvmResult {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("batch_all")),
        )
    }
    fn fallback(handle: &mut impl PrecompileHandle) -> EvmResult {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("fallback")),
        )
    }
}
#[allow(non_camel_case_types)]
pub enum BatchPrecompileCall<Runtime>
where
    Runtime: Get<u32>,
{
    batch_all {
        to: BoundedVec<Address, GetArrayLimit>,
        value: BoundedVec<U256, GetArrayLimit>,
        call_data: BoundedVec<BoundedBytes<GetCallDataLimit>, GetArrayLimit>,
        gas_limit: BoundedVec<u64, GetArrayLimit>,
    },
    batch_some {
        to: BoundedVec<Address, GetArrayLimit>,
        value: BoundedVec<U256, GetArrayLimit>,
        call_data: BoundedVec<BoundedBytes<GetCallDataLimit>, GetArrayLimit>,
        gas_limit: BoundedVec<u64, GetArrayLimit>,
    },
    batch_some_until_failure {
        to: BoundedVec<Address, GetArrayLimit>,
        value: BoundedVec<U256, GetArrayLimit>,
        call_data: BoundedVec<BoundedBytes<GetCallDataLimit>, GetArrayLimit>,
        gas_limit: BoundedVec<u64, GetArrayLimit>,
    },
    fallback {},
    #[doc(hidden)]
    __phantom(::core::marker::PhantomData<(Runtime)>, ::core::convert::Infallible),
}
impl<Runtime> BatchPrecompileCall<Runtime>
where
    Runtime: Get<u32>,
{
    pub fn parse_call_data(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::RevertReason;
        let input = handle.input();
        let selector = input
            .get(0..4)
            .map(|s| {
                let mut buffer = [0u8; 4];
                buffer.copy_from_slice(s);
                u32::from_be_bytes(buffer)
            });
        match selector {
            Some(2044677020u32) => Self::_parse_batch_some(handle),
            Some(2531431096u32) => Self::_parse_batch_all(handle),
            Some(3473183175u32) => Self::_parse_batch_some_until_failure(handle),
            _ => Self::_parse_fallback(handle),
        }
    }
    fn _parse_batch_all(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(4usize)?;
        Ok(Self::batch_all {
            to: input.read().in_field("to")?,
            value: input.read().in_field("value")?,
            call_data: input.read().in_field("callData")?,
            gas_limit: input.read().in_field("gasLimit")?,
        })
    }
    fn _parse_batch_some(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(4usize)?;
        Ok(Self::batch_some {
            to: input.read().in_field("to")?,
            value: input.read().in_field("value")?,
            call_data: input.read().in_field("callData")?,
            gas_limit: input.read().in_field("gasLimit")?,
        })
    }
    fn _parse_batch_some_until_failure(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(4usize)?;
        Ok(Self::batch_some_until_failure {
            to: input.read().in_field("to")?,
            value: input.read().in_field("value")?,
            call_data: input.read().in_field("callData")?,
            gas_limit: input.read().in_field("gasLimit")?,
        })
    }
    fn _parse_fallback(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        Ok(Self::fallback {})
    }
    pub fn execute(
        self,
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<::fp_evm::PrecompileOutput> {
        use ::precompile_utils::solidity::codec::Writer;
        use ::fp_evm::{PrecompileOutput, ExitSucceed};
        let output = match self {
            Self::batch_all { to, value, call_data, gas_limit } => {
                let output = <BatchPrecompile<
                    Runtime,
                >>::batch_all(handle, to, value, call_data, gas_limit);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::batch_some { to, value, call_data, gas_limit } => {
                let output = <BatchPrecompile<
                    Runtime,
                >>::batch_some(handle, to, value, call_data, gas_limit);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::batch_some_until_failure { to, value, call_data, gas_limit } => {
                let output = <BatchPrecompile<
                    Runtime,
                >>::batch_some_until_failure(handle, to, value, call_data, gas_limit);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::fallback {} => {
                let output = <BatchPrecompile<Runtime>>::fallback(handle);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::__phantom(_, _) => {
                ::core::panicking::panic_fmt(
                    format_args!("__phantom variant should not be used"),
                )
            }
        };
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output,
        })
    }
    pub fn supports_selector(selector: u32) -> bool {
        match selector {
            2044677020u32 => true,
            2531431096u32 => true,
            3473183175u32 => true,
            _ => false,
        }
    }
    pub fn selectors() -> &'static [u32] {
        &[2044677020u32, 2531431096u32, 3473183175u32]
    }
    pub fn batch_all_selectors() -> &'static [u32] {
        &[2531431096u32]
    }
    pub fn batch_some_selectors() -> &'static [u32] {
        &[2044677020u32]
    }
    pub fn batch_some_until_failure_selectors() -> &'static [u32] {
        &[3473183175u32]
    }
    pub fn fallback_selectors() -> &'static [u32] {
        &[]
    }
    pub fn encode(self) -> ::sp_std::vec::Vec<u8> {
        use ::precompile_utils::solidity::codec::Writer;
        match self {
            Self::batch_all { to, value, call_data, gas_limit } => {
                Writer::new_with_selector(2531431096u32)
                    .write(to)
                    .write(value)
                    .write(call_data)
                    .write(gas_limit)
                    .build()
            }
            Self::batch_some { to, value, call_data, gas_limit } => {
                Writer::new_with_selector(2044677020u32)
                    .write(to)
                    .write(value)
                    .write(call_data)
                    .write(gas_limit)
                    .build()
            }
            Self::batch_some_until_failure { to, value, call_data, gas_limit } => {
                Writer::new_with_selector(3473183175u32)
                    .write(to)
                    .write(value)
                    .write(call_data)
                    .write(gas_limit)
                    .build()
            }
            Self::fallback {} => Default::default(),
            Self::__phantom(_, _) => {
                ::core::panicking::panic_fmt(
                    format_args!("__phantom variant should not be used"),
                )
            }
        }
    }
}
impl<Runtime> From<BatchPrecompileCall<Runtime>> for ::sp_std::vec::Vec<u8>
where
    Runtime: Get<u32>,
{
    fn from(a: BatchPrecompileCall<Runtime>) -> ::sp_std::vec::Vec<u8> {
        a.encode()
    }
}
impl<Runtime> ::fp_evm::Precompile for BatchPrecompile<Runtime>
where
    Runtime: Get<u32>,
{
    fn execute(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<::fp_evm::PrecompileOutput> {
        let _: () = <BatchPrecompile<Runtime>>::pre_check(handle)?;
        <BatchPrecompileCall<Runtime>>::parse_call_data(handle)?.execute(handle)
    }
}
#[allow(non_snake_case)]
pub(crate) fn __BatchPrecompile_test_solidity_signatures_inner() {
    use ::precompile_utils::solidity::Codec;
    match (
        &"(address[],uint256[],bytes[],uint64[])",
        &<(
            BoundedVec<Address, GetArrayLimit>,
            BoundedVec<U256, GetArrayLimit>,
            BoundedVec<BoundedBytes<GetCallDataLimit>, GetArrayLimit>,
            BoundedVec<u64, GetArrayLimit>,
        ) as Codec>::signature(),
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::Some(
                        format_args!(
                            "{0} function signature doesn\'t match (left: attribute, right: computed from Rust types)",
                            "batch_all"
                        ),
                    ),
                );
            }
        }
    };
    match (
        &"(address[],uint256[],bytes[],uint64[])",
        &<(
            BoundedVec<Address, GetArrayLimit>,
            BoundedVec<U256, GetArrayLimit>,
            BoundedVec<BoundedBytes<GetCallDataLimit>, GetArrayLimit>,
            BoundedVec<u64, GetArrayLimit>,
        ) as Codec>::signature(),
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::Some(
                        format_args!(
                            "{0} function signature doesn\'t match (left: attribute, right: computed from Rust types)",
                            "batch_some"
                        ),
                    ),
                );
            }
        }
    };
    match (
        &"(address[],uint256[],bytes[],uint64[])",
        &<(
            BoundedVec<Address, GetArrayLimit>,
            BoundedVec<U256, GetArrayLimit>,
            BoundedVec<BoundedBytes<GetCallDataLimit>, GetArrayLimit>,
            BoundedVec<u64, GetArrayLimit>,
        ) as Codec>::signature(),
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::Some(
                        format_args!(
                            "{0} function signature doesn\'t match (left: attribute, right: computed from Rust types)",
                            "batch_some_until_failure"
                        ),
                    ),
                );
            }
        }
    };
    match (&"()", &<() as Codec>::signature()) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::Some(
                        format_args!(
                            "{0} function signature doesn\'t match (left: attribute, right: computed from Rust types)",
                            "fallback"
                        ),
                    ),
                );
            }
        }
    };
}
