use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_gain_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    let lifetimes = generics.lifetimes().filter(|l| l.lifetime.ident != "geo");
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
    let datagram = quote! {
        impl <'geo, #(#lifetimes,)* #(#type_params,)*> DatagramS<'geo> for #name #ty_generics #where_clause
        {
            type G = GainOperationGenerator<'geo, <Self as Gain<'geo>>::G>;
            type Error = GainError;

            fn operation_generator_with_segment(self, geometry: &'geo Geometry, env: &Environment, filter: &DeviceFilter, _: &FirmwareLimits, segment: Segment, transition_mode: Option<TransitionMode>) -> Result<Self::G, Self::Error> {
                Self::G::new(
                    self,
                    geometry,
                    env,
                    filter,
                    segment,
                    transition_mode
                )
            }

            fn option(&self) -> DatagramOption {
                DatagramOption {
                    parallel_threshold: num_cpus::get(),
                    ..DatagramOption::default()
                }
            }
        }
    };

    let lifetimes = generics.lifetimes().filter(|l| l.lifetime.ident != "geo");
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
    let inspect = quote! {
        impl <'geo, #(#lifetimes,)* #(#type_params,)*> Inspectable<'geo> for #name #ty_generics #where_clause
        {
            type Result = GainInspectionResult;

            fn inspect(
                self,
                geometry: &'geo Geometry,
                env: &Environment,
                filter: &DeviceFilter,
                _: &FirmwareLimits,
            ) -> Result<InspectionResult<GainInspectionResult>, GainError> {
                let mut g = self.init(geometry, env, &TransducerFilter::from(filter))?;
                let segment = Segment::S0;
                let transition_mode = None;
                Ok(InspectionResult::new(
                    geometry,
                    filter,
                    |dev| GainInspectionResult {
                            name: tynm::type_name::<Self>().to_string(),
                            data: {
                                let d = g.generate(dev);
                                dev.iter().map(|tr| d.calc(tr)).collect::<Vec<_>>()
                            },
                            segment,
                            transition_mode,
                    }
                ))
            }
        }
    };

    let generator = quote! {
        #datagram

        #inspect
    };
    generator.into()
}
