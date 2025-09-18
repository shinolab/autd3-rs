use proc_macro::TokenStream;
use quote::quote;

fn params(
    generics: &syn::Generics,
) -> (
    Vec<syn::Lifetime>,
    syn::TypeGenerics<'_>,
    proc_macro2::TokenStream,
    impl Iterator<Item = &syn::TypeParam>,
) {
    let lifetimes = generics
        .lifetimes()
        .filter(|l| l.lifetime.ident != "geo")
        .map(|l| l.lifetime.clone())
        .collect();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause = if let Some(w) = where_clause {
        quote! {
            #w
            Self: Gain<'geo>,
        }
    } else {
        quote! {
            where
                Self: Gain<'geo>,
        }
    };
    let type_params = generics.type_params();
    (lifetimes, ty_generics, where_clause, type_params)
}

pub(crate) fn impl_gain_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    let (lifetimes, ty_generics, where_clause, type_params) = params(generics);
    let datagram = quote! {
        impl <'geo, #(#lifetimes,)* #(#type_params,)*> DatagramS<'geo> for #name #ty_generics #where_clause
        {
            type G = GainOperationGenerator<'geo, <Self as Gain<'geo>>::G>;
            type Error = GainError;

            fn operation_generator_with_segment(self, geometry: &'geo Geometry, env: &Environment, filter: &DeviceMask, _: &FirmwareLimits, segment: Segment, transition_params: transition_mode::TransitionModeParams) -> Result<Self::G, Self::Error> {
                Self::G::new(
                    self,
                    geometry,
                    env,
                    filter,
                    segment,
                    transition_params,
                )
            }

            fn option(&self) -> DatagramOption {
                DatagramOption {
                    parallel_threshold: std::thread::available_parallelism().map(std::num::NonZeroUsize::get).unwrap_or(8),
                    ..DatagramOption::default()
                }
            }
        }
    };

    let (lifetimes, ty_generics, where_clause, type_params) = params(generics);
    let inspect = quote! {
        impl <'geo, #(#lifetimes,)* #(#type_params,)*> Inspectable<'geo> for #name #ty_generics #where_clause
        {
            type Result = GainInspectionResult;

            fn inspect(
                self,
                geometry: &'geo Geometry,
                env: &Environment,
                filter: &DeviceMask,
                _: &FirmwareLimits,
            ) -> Result<InspectionResult<GainInspectionResult>, GainError> {
                let mut g = self.init(geometry, env, &TransducerMask::from(filter))?;
                Ok(InspectionResult::new(
                    geometry,
                    filter,
                    |dev| GainInspectionResult {
                            data: {
                                let d = g.generate(dev);
                                dev.iter().map(|tr| d.calc(tr)).collect::<Vec<_>>()
                            },
                    }
                ))
            }
        }
    };

    let (lifetimes, ty_generics, where_clause, type_params) = params(generics);
    let segment_immediate = quote! {
        impl <'geo, #(#lifetimes,)* #(#type_params,)*> internal::HasSegment<transition_mode::Immediate> for #name #ty_generics #where_clause {}
    };

    let (lifetimes, ty_generics, where_clause, type_params) = params(generics);
    let segment_later = quote! {
        impl <'geo, #(#lifetimes,)* #(#type_params,)*> internal::HasSegment<transition_mode::Later> for #name #ty_generics #where_clause {}
    };

    let generator = quote! {
        #datagram

        #inspect

        #segment_immediate

        #segment_later
    };
    generator.into()
}
