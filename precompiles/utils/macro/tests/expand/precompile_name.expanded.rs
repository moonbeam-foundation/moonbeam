struct PrecompileAt<T, U, V = ()>(PhantomData<(T, U, V)>);
struct AddressU64<const N: u64>;
struct FooPrecompile<R>(PhantomData<R>);
struct BarPrecompile<R, S>(PhantomData<(R, S)>);
struct MockCheck;
type Precompiles = (
    PrecompileAt<AddressU64<1>, FooPrecompile<R>>,
    PrecompileAt<AddressU64<2>, BarPrecompile<R, S>, (MockCheck, MockCheck)>,
);
#[repr(u64)]
pub enum PrecompileName {
    FooPrecompile = 1u64,
    BarPrecompile = 2u64,
}
#[automatically_derived]
impl ::core::fmt::Debug for PrecompileName {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                PrecompileName::FooPrecompile => "FooPrecompile",
                PrecompileName::BarPrecompile => "BarPrecompile",
            },
        )
    }
}
impl PrecompileName {
    pub fn from_address(address: sp_core::H160) -> Option<Self> {
        let _u64 = address.to_low_u64_be();
        if address == sp_core::H160::from_low_u64_be(_u64) {
            use num_enum::TryFromPrimitive;
            Self::try_from_primitive(_u64).ok()
        } else {
            None
        }
    }
}
