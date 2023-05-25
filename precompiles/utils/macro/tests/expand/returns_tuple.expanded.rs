use {
    precompile_utils::{EvmResult, prelude::*},
    sp_core::{H160, U256},
};
struct ExamplePrecompile;
impl ExamplePrecompile {
    fn example(
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<(Address, U256, UnboundedBytes)> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("example")),
        )
    }
}
#[allow(non_camel_case_types)]
pub enum ExamplePrecompileCall {
    example {},
    #[doc(hidden)]
    __phantom(::core::marker::PhantomData<()>, ::core::convert::Infallible),
}
impl ExamplePrecompileCall {
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
            Some(1412775727u32) => Self::_parse_example(handle),
            Some(_) => Err(RevertReason::UnknownSelector.into()),
            None => Err(RevertReason::read_out_of_bounds("selector").into()),
        }
    }
    fn _parse_example(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        Ok(Self::example {})
    }
    pub fn execute(
        self,
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<::fp_evm::PrecompileOutput> {
        use ::precompile_utils::solidity::codec::Writer;
        use ::fp_evm::{PrecompileOutput, ExitSucceed};
        let output = match self {
            Self::example {} => {
                let output = <ExamplePrecompile>::example(handle);
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
            1412775727u32 => true,
            _ => false,
        }
    }
    pub fn selectors() -> &'static [u32] {
        &[1412775727u32]
    }
    pub fn example_selectors() -> &'static [u32] {
        &[1412775727u32]
    }
    pub fn encode(self) -> ::sp_std::vec::Vec<u8> {
        use ::precompile_utils::solidity::codec::Writer;
        match self {
            Self::example {} => Writer::new_with_selector(1412775727u32).build(),
            Self::__phantom(_, _) => {
                ::core::panicking::panic_fmt(
                    format_args!("__phantom variant should not be used"),
                )
            }
        }
    }
}
impl From<ExamplePrecompileCall> for ::sp_std::vec::Vec<u8> {
    fn from(a: ExamplePrecompileCall) -> ::sp_std::vec::Vec<u8> {
        a.encode()
    }
}
impl ::fp_evm::Precompile for ExamplePrecompile {
    fn execute(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<::fp_evm::PrecompileOutput> {
        <ExamplePrecompileCall>::parse_call_data(handle)?.execute(handle)
    }
}
#[allow(non_snake_case)]
pub(crate) fn __ExamplePrecompile_test_solidity_signatures_inner() {
    use ::precompile_utils::solidity::Codec;
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
                            "example"
                        ),
                    ),
                );
            }
        }
    };
}
