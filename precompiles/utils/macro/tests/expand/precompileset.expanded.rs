use {
    core::marker::PhantomData,
    precompile_utils::{EvmResult, prelude::*, testing::PrecompileTesterExt},
    sp_core::H160,
};
struct PrecompileSet<Runtime>(PhantomData<Runtime>);
type Discriminant = u32;
type GetAssetsStringLimit<R> = R;
type MockRuntime = ConstU32<42>;
impl<Runtime> PrecompileSet<Runtime>
where
    Runtime: Get<u32>,
{
    /// PrecompileSet discrimiant. Allows to knows if the address maps to an asset id,
    /// and if this is the case which one.
    fn discriminant(address: H160) -> Option<Discriminant> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("discriminant")),
        )
    }
    fn total_supply(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<U256> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("total_supply")),
        )
    }
    fn balance_of(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        who: Address,
    ) -> EvmResult<U256> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("balance_of")),
        )
    }
    fn allowance(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        owner: Address,
        spender: Address,
    ) -> EvmResult<U256> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("allowance")),
        )
    }
    fn approve(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        spender: Address,
        value: U256,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("approve")),
        )
    }
    fn approve_inner(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        owner: H160,
        spender: H160,
        value: U256,
    ) -> EvmResult {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("approve_inner")),
        )
    }
    fn transfer(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        to: Address,
        value: U256,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("transfer")),
        )
    }
    fn transfer_from(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        from: Address,
        to: Address,
        value: U256,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("transfer_from")),
        )
    }
    fn name(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<UnboundedBytes> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("name")),
        )
    }
    fn symbol(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<UnboundedBytes> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("symbol")),
        )
    }
    fn decimals(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<u8> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("decimals")),
        )
    }
    fn mint(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        to: Address,
        value: U256,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("mint")),
        )
    }
    fn burn(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        from: Address,
        value: U256,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("burn")),
        )
    }
    fn freeze(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        account: Address,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("freeze")),
        )
    }
    fn thaw(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        account: Address,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("thaw")),
        )
    }
    fn freeze_asset(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("freeze_asset")),
        )
    }
    fn thaw_asset(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("thaw_asset")),
        )
    }
    fn transfer_ownership(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        owner: Address,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("transfer_ownership")),
        )
    }
    fn set_team(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        issuer: Address,
        admin: Address,
        freezer: Address,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("set_team")),
        )
    }
    fn set_metadata(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        name: BoundedString<GetAssetsStringLimit<Runtime>>,
        symbol: BoundedString<GetAssetsStringLimit<Runtime>>,
        decimals: u8,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("set_metadata")),
        )
    }
    fn clear_metadata(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<bool> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("clear_metadata")),
        )
    }
    fn eip2612_permit(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        owner: Address,
        spender: Address,
        value: U256,
        deadline: U256,
        v: u8,
        r: H256,
        s: H256,
    ) -> EvmResult {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("eip2612_permit")),
        )
    }
    fn eip2612_nonces(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
        owner: Address,
    ) -> EvmResult<U256> {
        ::core::panicking::panic_fmt(
            format_args!("not yet implemented: {0}", format_args!("eip2612_nonces")),
        )
    }
    fn eip2612_domain_separator(
        asset_id: Discriminant,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<H256> {
        ::core::panicking::panic_fmt(
            format_args!(
                "not yet implemented: {0}", format_args!("eip2612_domain_separator")
            ),
        )
    }
}
#[allow(non_camel_case_types)]
pub enum PrecompileSetCall<Runtime>
where
    Runtime: Get<u32>,
{
    allowance { owner: Address, spender: Address },
    approve { spender: Address, value: U256 },
    balance_of { who: Address },
    burn { from: Address, value: U256 },
    clear_metadata {},
    decimals {},
    eip2612_domain_separator {},
    eip2612_nonces { owner: Address },
    eip2612_permit {
        owner: Address,
        spender: Address,
        value: U256,
        deadline: U256,
        v: u8,
        r: H256,
        s: H256,
    },
    freeze { account: Address },
    freeze_asset {},
    mint { to: Address, value: U256 },
    name {},
    set_metadata {
        name: BoundedString<GetAssetsStringLimit<Runtime>>,
        symbol: BoundedString<GetAssetsStringLimit<Runtime>>,
        decimals: u8,
    },
    set_team { issuer: Address, admin: Address, freezer: Address },
    symbol {},
    thaw { account: Address },
    thaw_asset {},
    total_supply {},
    transfer { to: Address, value: U256 },
    transfer_from { from: Address, to: Address, value: U256 },
    transfer_ownership { owner: Address },
    #[doc(hidden)]
    __phantom(::core::marker::PhantomData<(Runtime)>, ::core::convert::Infallible),
}
impl<Runtime> PrecompileSetCall<Runtime>
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
            Some(117300739u32) => Self::_parse_name(handle),
            Some(157198259u32) => Self::_parse_approve(handle),
            Some(404098525u32) => Self::_parse_total_supply(handle),
            Some(484305945u32) => Self::_parse_thaw_asset(handle),
            Some(599290589u32) => Self::_parse_transfer_from(handle),
            Some(826074471u32) => Self::_parse_decimals(handle),
            Some(910484757u32) => Self::_parse_eip2612_domain_separator(handle),
            Some(936559348u32) => Self::_parse_set_metadata(handle),
            Some(1086394137u32) => Self::_parse_mint(handle),
            Some(1374431959u32) => Self::_parse_thaw_asset(handle),
            Some(1587675670u32) => Self::_parse_thaw(handle),
            Some(1804030401u32) => Self::_parse_freeze_asset(handle),
            Some(1889567281u32) => Self::_parse_balance_of(handle),
            Some(2127478272u32) => Self::_parse_eip2612_nonces(handle),
            Some(2367676207u32) => Self::_parse_freeze(handle),
            Some(2514000705u32) => Self::_parse_symbol(handle),
            Some(2646777772u32) => Self::_parse_burn(handle),
            Some(2835717307u32) => Self::_parse_transfer(handle),
            Some(3352902745u32) => Self::_parse_set_team(handle),
            Some(3552201630u32) => Self::_parse_clear_metadata(handle),
            Some(3566436177u32) => Self::_parse_freeze_asset(handle),
            Some(3573918927u32) => Self::_parse_eip2612_permit(handle),
            Some(3714247998u32) => Self::_parse_allowance(handle),
            Some(3999121892u32) => Self::_parse_set_metadata(handle),
            Some(4021736498u32) => Self::_parse_clear_metadata(handle),
            Some(4030008324u32) => Self::_parse_transfer_ownership(handle),
            Some(4076725131u32) => Self::_parse_transfer_ownership(handle),
            Some(4173303445u32) => Self::_parse_set_team(handle),
            Some(_) => Err(RevertReason::UnknownSelector.into()),
            None => Err(RevertReason::read_out_of_bounds("selector").into()),
        }
    }
    fn _parse_allowance(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(2usize)?;
        Ok(Self::allowance {
            owner: input.read().in_field("owner")?,
            spender: input.read().in_field("spender")?,
        })
    }
    fn _parse_approve(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(2usize)?;
        Ok(Self::approve {
            spender: input.read().in_field("spender")?,
            value: input.read().in_field("value")?,
        })
    }
    fn _parse_balance_of(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(1usize)?;
        Ok(Self::balance_of {
            who: input.read().in_field("who")?,
        })
    }
    fn _parse_burn(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(2usize)?;
        Ok(Self::burn {
            from: input.read().in_field("from")?,
            value: input.read().in_field("value")?,
        })
    }
    fn _parse_clear_metadata(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        Ok(Self::clear_metadata {})
    }
    fn _parse_decimals(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        Ok(Self::decimals {})
    }
    fn _parse_eip2612_domain_separator(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::View)?;
        Ok(Self::eip2612_domain_separator {})
    }
    fn _parse_eip2612_nonces(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::View)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(1usize)?;
        Ok(Self::eip2612_nonces {
            owner: input.read().in_field("owner")?,
        })
    }
    fn _parse_eip2612_permit(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(7usize)?;
        Ok(Self::eip2612_permit {
            owner: input.read().in_field("owner")?,
            spender: input.read().in_field("spender")?,
            value: input.read().in_field("value")?,
            deadline: input.read().in_field("deadline")?,
            v: input.read().in_field("v")?,
            r: input.read().in_field("r")?,
            s: input.read().in_field("s")?,
        })
    }
    fn _parse_freeze(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(1usize)?;
        Ok(Self::freeze {
            account: input.read().in_field("account")?,
        })
    }
    fn _parse_freeze_asset(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        Ok(Self::freeze_asset {})
    }
    fn _parse_mint(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(2usize)?;
        Ok(Self::mint {
            to: input.read().in_field("to")?,
            value: input.read().in_field("value")?,
        })
    }
    fn _parse_name(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        Ok(Self::name {})
    }
    fn _parse_set_metadata(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(3usize)?;
        Ok(Self::set_metadata {
            name: input.read().in_field("name")?,
            symbol: input.read().in_field("symbol")?,
            decimals: input.read().in_field("decimals")?,
        })
    }
    fn _parse_set_team(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(3usize)?;
        Ok(Self::set_team {
            issuer: input.read().in_field("issuer")?,
            admin: input.read().in_field("admin")?,
            freezer: input.read().in_field("freezer")?,
        })
    }
    fn _parse_symbol(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        Ok(Self::symbol {})
    }
    fn _parse_thaw(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(1usize)?;
        Ok(Self::thaw {
            account: input.read().in_field("account")?,
        })
    }
    fn _parse_thaw_asset(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        Ok(Self::thaw_asset {})
    }
    fn _parse_total_supply(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        Ok(Self::total_supply {})
    }
    fn _parse_transfer(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(2usize)?;
        Ok(Self::transfer {
            to: input.read().in_field("to")?,
            value: input.read().in_field("value")?,
        })
    }
    fn _parse_transfer_from(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(3usize)?;
        Ok(Self::transfer_from {
            from: input.read().in_field("from")?,
            to: input.read().in_field("to")?,
            value: input.read().in_field("value")?,
        })
    }
    fn _parse_transfer_ownership(
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<Self> {
        use ::precompile_utils::solidity::revert::InjectBacktrace;
        use ::precompile_utils::solidity::modifier::FunctionModifier;
        use ::precompile_utils::evm::handle::PrecompileHandleExt;
        handle.check_function_modifier(FunctionModifier::NonPayable)?;
        let mut input = handle.read_after_selector()?;
        input.expect_arguments(1usize)?;
        Ok(Self::transfer_ownership {
            owner: input.read().in_field("owner")?,
        })
    }
    pub fn execute(
        self,
        discriminant: Discriminant,
        handle: &mut impl PrecompileHandle,
    ) -> ::precompile_utils::EvmResult<::fp_evm::PrecompileOutput> {
        use ::precompile_utils::solidity::codec::Writer;
        use ::fp_evm::{PrecompileOutput, ExitSucceed};
        let output = match self {
            Self::allowance { owner, spender } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::allowance(discriminant, handle, owner, spender);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::approve { spender, value } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::approve(discriminant, handle, spender, value);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::balance_of { who } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::balance_of(discriminant, handle, who);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::burn { from, value } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::burn(discriminant, handle, from, value);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::clear_metadata {} => {
                let output = <PrecompileSet<
                    Runtime,
                >>::clear_metadata(discriminant, handle);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::decimals {} => {
                let output = <PrecompileSet<Runtime>>::decimals(discriminant, handle);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::eip2612_domain_separator {} => {
                let output = <PrecompileSet<
                    Runtime,
                >>::eip2612_domain_separator(discriminant, handle);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::eip2612_nonces { owner } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::eip2612_nonces(discriminant, handle, owner);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::eip2612_permit { owner, spender, value, deadline, v, r, s } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::eip2612_permit(
                    discriminant,
                    handle,
                    owner,
                    spender,
                    value,
                    deadline,
                    v,
                    r,
                    s,
                );
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::freeze { account } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::freeze(discriminant, handle, account);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::freeze_asset {} => {
                let output = <PrecompileSet<
                    Runtime,
                >>::freeze_asset(discriminant, handle);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::mint { to, value } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::mint(discriminant, handle, to, value);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::name {} => {
                let output = <PrecompileSet<Runtime>>::name(discriminant, handle);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::set_metadata { name, symbol, decimals } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::set_metadata(discriminant, handle, name, symbol, decimals);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::set_team { issuer, admin, freezer } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::set_team(discriminant, handle, issuer, admin, freezer);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::symbol {} => {
                let output = <PrecompileSet<Runtime>>::symbol(discriminant, handle);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::thaw { account } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::thaw(discriminant, handle, account);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::thaw_asset {} => {
                let output = <PrecompileSet<Runtime>>::thaw_asset(discriminant, handle);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::total_supply {} => {
                let output = <PrecompileSet<
                    Runtime,
                >>::total_supply(discriminant, handle);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::transfer { to, value } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::transfer(discriminant, handle, to, value);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::transfer_from { from, to, value } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::transfer_from(discriminant, handle, from, to, value);
                ::precompile_utils::solidity::encode_return_value(output?)
            }
            Self::transfer_ownership { owner } => {
                let output = <PrecompileSet<
                    Runtime,
                >>::transfer_ownership(discriminant, handle, owner);
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
            117300739u32 => true,
            157198259u32 => true,
            404098525u32 => true,
            484305945u32 => true,
            599290589u32 => true,
            826074471u32 => true,
            910484757u32 => true,
            936559348u32 => true,
            1086394137u32 => true,
            1374431959u32 => true,
            1587675670u32 => true,
            1804030401u32 => true,
            1889567281u32 => true,
            2127478272u32 => true,
            2367676207u32 => true,
            2514000705u32 => true,
            2646777772u32 => true,
            2835717307u32 => true,
            3352902745u32 => true,
            3552201630u32 => true,
            3566436177u32 => true,
            3573918927u32 => true,
            3714247998u32 => true,
            3999121892u32 => true,
            4021736498u32 => true,
            4030008324u32 => true,
            4076725131u32 => true,
            4173303445u32 => true,
            _ => false,
        }
    }
    pub fn selectors() -> &'static [u32] {
        &[
            117300739u32,
            157198259u32,
            404098525u32,
            484305945u32,
            599290589u32,
            826074471u32,
            910484757u32,
            936559348u32,
            1086394137u32,
            1374431959u32,
            1587675670u32,
            1804030401u32,
            1889567281u32,
            2127478272u32,
            2367676207u32,
            2514000705u32,
            2646777772u32,
            2835717307u32,
            3352902745u32,
            3552201630u32,
            3566436177u32,
            3573918927u32,
            3714247998u32,
            3999121892u32,
            4021736498u32,
            4030008324u32,
            4076725131u32,
            4173303445u32,
        ]
    }
    pub fn allowance_selectors() -> &'static [u32] {
        &[3714247998u32]
    }
    pub fn approve_selectors() -> &'static [u32] {
        &[157198259u32]
    }
    pub fn balance_of_selectors() -> &'static [u32] {
        &[1889567281u32]
    }
    pub fn burn_selectors() -> &'static [u32] {
        &[2646777772u32]
    }
    pub fn clear_metadata_selectors() -> &'static [u32] {
        &[4021736498u32, 3552201630u32]
    }
    pub fn decimals_selectors() -> &'static [u32] {
        &[826074471u32]
    }
    pub fn eip2612_domain_separator_selectors() -> &'static [u32] {
        &[910484757u32]
    }
    pub fn eip2612_nonces_selectors() -> &'static [u32] {
        &[2127478272u32]
    }
    pub fn eip2612_permit_selectors() -> &'static [u32] {
        &[3573918927u32]
    }
    pub fn freeze_selectors() -> &'static [u32] {
        &[2367676207u32]
    }
    pub fn freeze_asset_selectors() -> &'static [u32] {
        &[3566436177u32, 1804030401u32]
    }
    pub fn mint_selectors() -> &'static [u32] {
        &[1086394137u32]
    }
    pub fn name_selectors() -> &'static [u32] {
        &[117300739u32]
    }
    pub fn set_metadata_selectors() -> &'static [u32] {
        &[936559348u32, 3999121892u32]
    }
    pub fn set_team_selectors() -> &'static [u32] {
        &[3352902745u32, 4173303445u32]
    }
    pub fn symbol_selectors() -> &'static [u32] {
        &[2514000705u32]
    }
    pub fn thaw_selectors() -> &'static [u32] {
        &[1587675670u32]
    }
    pub fn thaw_asset_selectors() -> &'static [u32] {
        &[1374431959u32, 484305945u32]
    }
    pub fn total_supply_selectors() -> &'static [u32] {
        &[404098525u32]
    }
    pub fn transfer_selectors() -> &'static [u32] {
        &[2835717307u32]
    }
    pub fn transfer_from_selectors() -> &'static [u32] {
        &[599290589u32]
    }
    pub fn transfer_ownership_selectors() -> &'static [u32] {
        &[4076725131u32, 4030008324u32]
    }
    pub fn encode(self) -> ::sp_std::vec::Vec<u8> {
        use ::precompile_utils::solidity::codec::Writer;
        match self {
            Self::allowance { owner, spender } => {
                Writer::new_with_selector(3714247998u32)
                    .write(owner)
                    .write(spender)
                    .build()
            }
            Self::approve { spender, value } => {
                Writer::new_with_selector(157198259u32)
                    .write(spender)
                    .write(value)
                    .build()
            }
            Self::balance_of { who } => {
                Writer::new_with_selector(1889567281u32).write(who).build()
            }
            Self::burn { from, value } => {
                Writer::new_with_selector(2646777772u32).write(from).write(value).build()
            }
            Self::clear_metadata {} => Writer::new_with_selector(4021736498u32).build(),
            Self::decimals {} => Writer::new_with_selector(826074471u32).build(),
            Self::eip2612_domain_separator {} => {
                Writer::new_with_selector(910484757u32).build()
            }
            Self::eip2612_nonces { owner } => {
                Writer::new_with_selector(2127478272u32).write(owner).build()
            }
            Self::eip2612_permit { owner, spender, value, deadline, v, r, s } => {
                Writer::new_with_selector(3573918927u32)
                    .write(owner)
                    .write(spender)
                    .write(value)
                    .write(deadline)
                    .write(v)
                    .write(r)
                    .write(s)
                    .build()
            }
            Self::freeze { account } => {
                Writer::new_with_selector(2367676207u32).write(account).build()
            }
            Self::freeze_asset {} => Writer::new_with_selector(3566436177u32).build(),
            Self::mint { to, value } => {
                Writer::new_with_selector(1086394137u32).write(to).write(value).build()
            }
            Self::name {} => Writer::new_with_selector(117300739u32).build(),
            Self::set_metadata { name, symbol, decimals } => {
                Writer::new_with_selector(936559348u32)
                    .write(name)
                    .write(symbol)
                    .write(decimals)
                    .build()
            }
            Self::set_team { issuer, admin, freezer } => {
                Writer::new_with_selector(3352902745u32)
                    .write(issuer)
                    .write(admin)
                    .write(freezer)
                    .build()
            }
            Self::symbol {} => Writer::new_with_selector(2514000705u32).build(),
            Self::thaw { account } => {
                Writer::new_with_selector(1587675670u32).write(account).build()
            }
            Self::thaw_asset {} => Writer::new_with_selector(1374431959u32).build(),
            Self::total_supply {} => Writer::new_with_selector(404098525u32).build(),
            Self::transfer { to, value } => {
                Writer::new_with_selector(2835717307u32).write(to).write(value).build()
            }
            Self::transfer_from { from, to, value } => {
                Writer::new_with_selector(599290589u32)
                    .write(from)
                    .write(to)
                    .write(value)
                    .build()
            }
            Self::transfer_ownership { owner } => {
                Writer::new_with_selector(4076725131u32).write(owner).build()
            }
            Self::__phantom(_, _) => {
                ::core::panicking::panic_fmt(
                    format_args!("__phantom variant should not be used"),
                )
            }
        }
    }
}
impl<Runtime> From<PrecompileSetCall<Runtime>> for ::sp_std::vec::Vec<u8>
where
    Runtime: Get<u32>,
{
    fn from(a: PrecompileSetCall<Runtime>) -> ::sp_std::vec::Vec<u8> {
        a.encode()
    }
}
impl<Runtime> ::fp_evm::PrecompileSet for PrecompileSet<Runtime>
where
    Runtime: Get<u32>,
{
    fn execute(
        &self,
        handle: &mut impl PrecompileHandle,
    ) -> Option<::precompile_utils::EvmResult<::fp_evm::PrecompileOutput>> {
        let discriminant = match <PrecompileSet<
            Runtime,
        >>::discriminant(handle.code_address()) {
            Some(d) => d,
            None => return None,
        };
        Some(
            <PrecompileSetCall<Runtime>>::parse_call_data(handle)
                .and_then(|call| call.execute(discriminant, handle)),
        )
    }
    fn is_precompile(&self, address: H160, gas: u64) -> ::fp_evm::IsPrecompileResult {
        <PrecompileSet<Runtime>>::discriminant(address, gas).is_some()
    }
}
#[allow(non_snake_case)]
pub(crate) fn __PrecompileSet_test_solidity_signatures_inner<Runtime>()
where
    Runtime: Get<u32>,
{
    use ::precompile_utils::solidity::Codec;
    match (&"(address,address)", &<(Address, Address) as Codec>::signature()) {
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
                            "allowance"
                        ),
                    ),
                );
            }
        }
    };
    match (&"(address,uint256)", &<(Address, U256) as Codec>::signature()) {
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
                            "approve"
                        ),
                    ),
                );
            }
        }
    };
    match (&"(address)", &<(Address,) as Codec>::signature()) {
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
                            "balance_of"
                        ),
                    ),
                );
            }
        }
    };
    match (&"(address,uint256)", &<(Address, U256) as Codec>::signature()) {
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
                            "burn"
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
                            "clear_metadata"
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
                            "decimals"
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
                            "eip2612_domain_separator"
                        ),
                    ),
                );
            }
        }
    };
    match (&"(address)", &<(Address,) as Codec>::signature()) {
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
                            "eip2612_nonces"
                        ),
                    ),
                );
            }
        }
    };
    match (
        &"(address,address,uint256,uint256,uint8,bytes32,bytes32)",
        &<(Address, Address, U256, U256, u8, H256, H256) as Codec>::signature(),
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
                            "eip2612_permit"
                        ),
                    ),
                );
            }
        }
    };
    match (&"(address)", &<(Address,) as Codec>::signature()) {
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
                            "freeze"
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
                            "freeze_asset"
                        ),
                    ),
                );
            }
        }
    };
    match (&"(address,uint256)", &<(Address, U256) as Codec>::signature()) {
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
                            "mint"
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
                            "name"
                        ),
                    ),
                );
            }
        }
    };
    match (
        &"(string,string,uint8)",
        &<(
            BoundedString<GetAssetsStringLimit<Runtime>>,
            BoundedString<GetAssetsStringLimit<Runtime>>,
            u8,
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
                            "set_metadata"
                        ),
                    ),
                );
            }
        }
    };
    match (
        &"(address,address,address)",
        &<(Address, Address, Address) as Codec>::signature(),
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
                            "set_team"
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
                            "symbol"
                        ),
                    ),
                );
            }
        }
    };
    match (&"(address)", &<(Address,) as Codec>::signature()) {
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
                            "thaw"
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
                            "thaw_asset"
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
                            "total_supply"
                        ),
                    ),
                );
            }
        }
    };
    match (&"(address,uint256)", &<(Address, U256) as Codec>::signature()) {
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
                            "transfer"
                        ),
                    ),
                );
            }
        }
    };
    match (
        &"(address,address,uint256)",
        &<(Address, Address, U256) as Codec>::signature(),
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
                            "transfer_from"
                        ),
                    ),
                );
            }
        }
    };
    match (&"(address)", &<(Address,) as Codec>::signature()) {
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
                            "transfer_ownership"
                        ),
                    ),
                );
            }
        }
    };
}
