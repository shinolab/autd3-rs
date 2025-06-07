use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_gain_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    let lifetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let datagram = quote! {
        impl <#(#lifetimes,)* #(#type_params,)*> DatagramS for #name #ty_generics #where_clause
        {
            type G = GainOperationGenerator<<Self as Gain>::G>;
            type Error = GainError;

            fn operation_generator_with_segment(self, geometry: &Geometry, filter: &DeviceFilter, segment: Segment, transition_mode: Option<TransitionMode>) -> Result<Self::G, Self::Error> {
                Self::G::new(
                    self,
                    geometry,
                    filter,
                    segment,
                    transition_mode
                )
            }

            fn option(&self) -> DatagramOption {
                DatagramOption {
                    timeout: std::time::Duration::from_millis(20),
                    parallel_threshold: num_cpus::get(),
                }
            }
        }
    };

    let lifetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let inspect = quote! {
        impl <#(#lifetimes,)* #(#type_params,)*> Inspectable for #name #ty_generics #where_clause {
            type Result = GainInspectionResult;

            fn inspect(
                self,
                geometry: &Geometry,
                filter: &DeviceFilter,
            ) -> Result<InspectionResult<Self::Result>, Self::Error> {
                let mut g = self.init(geometry, &TransducerFilter::from(filter))?;
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
