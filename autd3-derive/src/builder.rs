use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::Meta;

fn impl_getter(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let generics = &input.generics;

    let getter_fileds = if let syn::Data::Struct(syn::DataStruct { fields, .. }) =
        input.data.clone()
    {
        fields
                .iter()
                .filter_map(|field| {
                    if field.attrs.iter().any(
                        |attr| matches!(&attr.meta, Meta::Path(path) if path.is_ident("get") || path.is_ident("getset")),
                    ) {
                        Some(field.clone())
                    } else {
                        None
                    }
                })
                .collect()
    } else {
        vec![]
    };

    let getters = getter_fileds.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        quote! {
            pub const fn #ident(&self) -> #ty {
                self.#ident
            }
        }
    });

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl <#(#linetimes,)* #(#type_params,)*> #name #ty_generics #where_clause {
           #(#getters)*
        }
    }
}

fn impl_setter(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let generics = &input.generics;

    let setter_fileds = if let syn::Data::Struct(syn::DataStruct { fields, .. }) =
        input.data.clone()
    {
        fields
                .iter()
                .filter_map(|field| {
                    if field.attrs.iter().any(
                        |attr| matches!(&attr.meta, Meta::Path(path) if path.is_ident("set") || path.is_ident("getset")),
                    ) {
                        Some(field.clone())
                    } else {
                        None
                    }
                })
                .collect()
    } else {
        vec![]
    };

    let setters = setter_fileds.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let name = format_ident!("with_{}", ident);
        match ty {
            syn::Type::Path(path) if path.path.is_ident("EmitIntensity") => quote! {
                pub fn #name(mut self, value: impl Into<EmitIntensity>) -> Self {
                    self.#ident = value.into();
                    self
                }
            },
            _ => quote! {
                pub fn #name(mut self, value: #ty) -> Self {
                    self.#ident = value;
                    self
                }
            },
        }
    });

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl <#(#linetimes,)* #(#type_params,)*> #name #ty_generics #where_clause {
           #(#setters)*
        }
    }
}

pub(crate) fn impl_builder_macro(input: syn::DeriveInput) -> TokenStream {
    let getters = impl_getter(&input);
    let setters = impl_setter(&input);

    let gen = quote! {
        #getters

        #setters
    };
    gen.into()
}
