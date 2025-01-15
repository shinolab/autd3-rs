#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! # A custom derive macro for `autd3`.

mod builder;
mod gain;
mod modulation;

use proc_macro::TokenStream;

/// A macro to define a custom [`Gain`].
///
/// # Example
///
/// The following example shows how to define a custom [`Gain`] that generates a single focal point.
///
/// ```
/// use autd3_core::derive::*;
/// use autd3_core::geometry::Point3;
/// use autd3_core::defined::rad;
///
/// #[derive(Gain, Debug)]
/// pub struct FocalPoint {
///     pos: Point3,
/// }
///
/// pub struct Context {
///     pos: Point3,
///     wavenumber: f32,
/// }
///
/// impl GainContext for Context {
///     fn calc(&self, tr: &Transducer) -> Drive {
///         (
///             Phase::from(-(self.pos - tr.position()).norm() * self.wavenumber * rad),
///             EmitIntensity::MAX,
///         )
///             .into()
///     }
/// }
///
/// impl GainContextGenerator for FocalPoint {
///     type Context = Context;
///
///     fn generate(&mut self, device: &Device) -> Self::Context {
///         Context {
///             pos: self.pos,
///             wavenumber: device.wavenumber(),
///         }
///     }
/// }
///
/// impl Gain for FocalPoint {
///     type G = FocalPoint;
///
///     fn init(
///         self,
///         _geometry: &Geometry,
///         _filter: Option<&HashMap<usize, BitVec>>,
///     ) -> Result<Self::G, GainError> {
///         Ok(self)
///     }
/// }
/// ```
///
/// [`Gain`]: https://docs.rs/autd3-core/latest/autd3_core/gain/trait.Gain.html
#[proc_macro_derive(Gain)]
pub fn gain_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    gain::impl_gain_macro(ast)
}

/// A macro to define a custom [`Modulation`].
///
/// # Example
///
/// The following example shows how to define a modulation that outputs the maximum value only for a moment.
///
/// [`Modulation`] struct must have `config: SamplingConfig` and `loop_behavior: LoopBehavior`.
/// If you add `#[no_change]` attribute to `config`, you can't change the value of `config` except for the constructor.
///
/// ```
/// use autd3_core::derive::*;
///
/// #[derive(Modulation, Debug)]
/// pub struct Burst {
///     config: SamplingConfig,
///     loop_behavior: LoopBehavior,
/// }
///
/// impl Burst {
///     pub fn new() -> Self {
///         Self {
///             config: SamplingConfig::FREQ_4K,
///             loop_behavior: LoopBehavior::infinite(),
///         }
///     }
/// }
///
/// impl Modulation for Burst {
///     fn calc(self) -> Result<Vec<u8>, ModulationError>  {
///         Ok((0..4000)
///             .map(|i| if i == 3999 { u8::MAX } else { u8::MIN })
///             .collect())
///     }
/// }
/// ```
///
/// [`Modulation`]: https://docs.rs/autd3-core/latest/autd3_core/gain/trait.Modulation.html
#[proc_macro_derive(Modulation, attributes(no_change))]
pub fn modulation_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    modulation::impl_mod_macro(ast)
}

/// A trait for builder pattern.
///
///
/// # Example
///
/// ## getter
///
/// ```
/// use autd3_derive::Builder;
///
/// #[derive(Builder)]
/// struct Foo {
///     #[get]
///     value: i32
/// }
/// ```
///
/// will generate the following code:
///
/// ```ignore
/// impl Foo {
///     #[must_use]
///     pub const fn value(&self) -> i32 {
///         self.value
///     }
/// }
/// ```
///
/// If you use `get(ref)` and `get(ref_mut)`, you can get the reference of the field.
///
/// ```
/// use autd3_derive::Builder;
///
/// #[derive(Builder)]
/// struct Foo {
///     #[get(ref, ref_mut)]
///     value: i32,
/// }
/// ```
///
/// will generate the following code:
///
/// ```ignore
/// impl Foo {
///     #[must_use]
///     pub const fn value(&self) -> &i32 {
///         &self.value
///     }
///     #[must_use]
///     pub fn value_mut(&mut self) -> &mut i32 {
///         &mut self.value
///     }
/// }
/// ```
///
/// ## setter
///
/// ```
/// use autd3_derive::Builder;
///
/// #[derive(Builder)]
/// struct Foo {
///     #[set]
///     value: i32,
/// }
/// ```
///
/// will generate the following code:
///
/// ```ignore
/// impl Foo {
///     #[allow(clippy::needless_update)]
///     #[must_use]
///     pub const fn with_value(mut self, value: i32) -> Self {
///         self.value = value;
///         self
///     }
/// }
/// ```
///
/// If you use `set(into)`, you can use `Into` trait for the setter.
///
/// ```
/// use autd3_derive::Builder;
///
/// #[derive(Builder)]
/// struct Foo {
///     #[set(into)]
///     value: i32,
/// }
/// ```
///
/// will generate the following code:
///
/// ```ignore
/// impl Foo {
///     #[allow(clippy::needless_update)]
///     #[must_use]
///     pub fn with_value(mut self, value: impl Into<i32>) -> Self {
///         self.value = value.into();
///         self
///     }
/// }
/// ```
#[proc_macro_derive(Builder, attributes(get, set))]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    builder::impl_builder_macro(ast)
}
