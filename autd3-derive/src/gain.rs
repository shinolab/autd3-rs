use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_gain_option_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    let lifetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let gain_option = quote! {
        impl <#(#lifetimes,)* #(#type_params,)*> GainOption for #name #ty_generics #where_clause
        {
            fn segment(&self) -> Segment {
                self.segment
            }

            fn transition_mode(&self) -> Option<TransitionMode> {
                self.transition_mode
            }
        }
    };

    let generator = quote! {
        #gain_option
    };
    generator.into()
}

pub(crate) fn impl_gain_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    let lifetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let option = quote! {
        impl <#(#lifetimes,)* #(#type_params,)*> GetGainOption for #name #ty_generics #where_clause
        {
            fn option(&self) -> &<Self as Gain>::Option {
                &self.option
            }
        }
    };

    let lifetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let datagram = quote! {
        impl <#(#lifetimes,)* #(#type_params,)*> Datagram for #name #ty_generics #where_clause
        {
            type G = GainOperationGenerator<<Self as Gain>::G>;
            type Error = GainError;

            fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, Self::Error> {
                let option = <Self as GetGainOption>::option(&self);
                let segment = option.segment();
                let transition_mode = option.transition_mode();
                Self::G::new(
                    self,
                    geometry,
                    segment,
                    transition_mode,
                )
            }

            fn timeout(&self) -> Option<std::time::Duration> {
                None
            }

            fn parallel_threshold(&self) -> Option<usize> {
                None
            }
        }
    };

    let generator = quote! {
        #datagram

        #option
    };
    generator.into()
}
