# `#[precompile]` procedural macro.

This procedural macro allows to simplify the implementation of an EVM precompile or precompile set
using an `impl` block with annotations to automatically generate:

- the implementation of the trait `Precompile` or `PrecompileSet` (exposed by the `fp_evm` crate)
- parsing of the method parameters from Solidity encoding into Rust type, based on the `solidity::Codec`
  trait (exposed by the `precompile-utils` crate)
- a test to ensure the types expressed in the Solidity signature match the Rust types in the
  implementation.

## How to use

Define your precompile type and write an `impl` block that will contain the precompile methods
implementation. This `impl` block can have type parameters and a `where` clause, which will be
reused to generate the `Precompile`/`PrecompileSet` trait implementation and the enum representing
each public function of precompile with its parsed arguments.

```rust,ignore
pub struct ExemplePrecompile<R, I>(PhantomData<(R,I)>);

#[precomile_utils::precompile]
impl<R, I> ExemplePrecompile<R, I>
where
    R: pallet_evm::Config
{
    #[precompile::public("example(uint32)")]
    fn example(handle: &mut impl PrecompileHandle, arg: u32) -> EvmResult<u32> {
        Ok(arg * 2)
    }
}
```

The example code above will automatically generate an enum like

```rust,ignore
#[allow(non_camel_case_types)]
pub enum ExemplePrecompileCall<R, I>
where
    R: pallet_evm::Config
{
    example {
        arg: u32
    },
    // + an non constrible variant with a PhantomData<(R,I)>
}
```

This enum have the function `parse_call_data` that can parse the calldata, recognize the Solidity
4-bytes selector and parse the appropriate enum variant.

It will also generate automatically an implementation of `Precompile`/`PrecompileSet` that calls
this function and the content of the variant to its associated function of the `impl` block.

## Function attributes

`#[precompile::public("signature")]` allows to declare a function as a public method of the
precompile with the provided Solidity signature. A function can have multiple `public` attributes to
support renamed functions with backward compatibility, however the arguments must have the same
type. It is not allowed to use the exact same signature multiple times.

The function must take a `&mut impl PrecompileHandle` as parameter, followed by all the parameters
of the Solidity function in the same order. Those parameters types must implement `solidity::Codec`, and
their name should match the one used in the Solidity interface (.sol) while being in `snake_case`,
which will automatically be converted to `camelCase` in revert messages. The function must return an
`EvmResult<T>`, which is an alias of `Result<T, PrecompileFailure>`. This `T` must implement the
`solidity::Codec` trait and must match the return type in the Solidity interface. The macro will
automatically encode it to Solidity format.

By default those functions are considered non-payable and non-view (can cause state changes). This
can be changed using either `#[precompile::payable]` or `#[precompile::view]`. Only one can be used.

It is also possible to declare a fallback function using `#[precompile::fallback]`. This function
will be called if the selector is unknown or if the input is less than 4-bytes long (no selector).
This function cannot have any parameter outside of the `PrecompileHandle`. A function can be both
`public` and `fallback`.

In case some check must be performed before parsing the input, such as forbidding being called from
some address, a function can be annotated with `#[precompile::pre_check]`:

```rust,ignore
#[precompile::pre_check]
fn pre_check(handle: &mut impl PrecompileHandle) -> EvmResult {
    todo!("Perform your check here")
}
```

This function cannot have other attributes.

## PrecompileSet

By default the macro considers the `impl` block to represent a precompile and this will implement
the `Precompile` trait. If you want to instead implement a precompile set, you must add the
`#[precompile::precompile_set]` to the `impl` block.

Then, it is necessary to have a function annotated with the `#[precompile::discriminant]` attribute.
This function is called with the **code address**, the address of the precompile. It must return
`None` if this address is not part of the precompile set, or `Some` if it is. The `Some` variants
contains a value of a type of your choice that represents which member of the set this address
corresponds to. For example for our XC20 precompile sets this function returns the asset id
corresponding to this address if it exists.

Finally, every other function annotated with a `precompile::_` attribute must now take this
discriminant as first parameter, before the `PrecompileHandle`.

```rust,ignore
pub struct ExemplePrecompileSet<R>(PhantomData<R>);

#[precompile_utils::precompile]
#[precompile::precompile_set]
impl<R> ExamplePrecompileSet<R>
where
    R: pallet_evm::Config
{
    #[precompile::discriminant]
    fn discriminant(address: H160) -> Option<u8> {
        // Replace with your discriminant logic.
        Some(match address {
            a if a == H160::from(42) => 1
            a if a == H160::from(43) => 2,
            _ => return None,
        })
    }

    #[precompile::public("example(uint32)")]
    fn example(discriminant: u8, handle: &mut impl PrecompileHandle, arg: u32) -> EvmResult {
        // Discriminant can be used here.
        Ok(arg * discriminant)
    }
}
```

## Solidity signatures test

The macro will automatically generate a unit test to ensure that the types expressed in a `public`
attribute matches the Rust parameters of the function, thanks to the `solidity::Codec` trait having the
`solidity_type() -> String` function.

If any **parsed** argument (discriminant is not concerned) depends on the type parameters of the
`impl` block, the macro will not be able to produce valid code and output an error like:

```text
error[E0412]: cannot find type `R` in this scope
  --> tests/precompile/compile-fail/test/generic-arg.rs:25:63
   |
23 | impl<R: Get<u32>> Precompile<R> {
   |                             - help: you might be missing a type parameter: `<R>`
24 |     #[precompile::public("foo(bytes)")]
25 |     fn foo(handle: &mut impl PrecompileHandle, arg: BoundedBytes<R>) -> EvmResult {
   |                                                                  ^ not found in this scope
```

In this case you need to annotate the `impl` block with the `#[precompile::test_concrete_types(...)]`
attributes. The `...` should be replaced with concrete types for each type parameter, like a mock
runtime. Those types are only used to generate the test and only one set of types can be used.

```rust,ignore
pub struct ExamplePrecompile<R, I>(PhantomData<(R, I)>);

pub struct GetMaxSize<R, I>(PhantomData<(R, I)>);

impl<R: SomeConfig, I> Get<u32> for GetMaxSize<R, I> {
	fn get() -> u32 {
		<R as SomeConfig<I>>::SomeConstant::get()
	}
}

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime, Instance1)]
impl<R, I> ExamplePrecompile<R, I>
where
	R: pallet_evm::Config + SomeConfig<I>
{
	#[precompile::public("example(bytes)")]
	fn example(
		handle: &mut impl PrecompileHandle,
		data: BoundedBytes<GetMaxSize<R>>,
	) -> EvmResult {
		todo!("Method implementation")
	}
}
```

## Enum functions

The generated enums exposes the following public functions:

- `parse_call_data`: take a `PrecompileHandle` and tries to parse the call data. Returns an
  `EvmResult<Self>`. It **DOES NOT** execute the code of the annotated `impl` block.
- `supports_selector`: take a selector as a `u32` is returns if this selector is supported by the
  precompile(set) as a `bool`. Note that the presence of a fallback function is not taken into
  account.
- `selectors`: returns a static array (`&'static [u32]`) of all the supported selectors.
- For each variant/public function `foo`, there is a function `foo_selectors` which returns a static
  array of all the supported selectors **for that function**. That can be used to ensure in tests
  that some function have a selector that was computed by hand.
- `encode`: take `self` and encodes it in Solidity format. Additionally, `Vec<u8>` implements
  `From<CallEnum>` which simply call encodes. This is useful to write tests as you can construct the
  variant you want and it will be encoded to Solidity format for you.
