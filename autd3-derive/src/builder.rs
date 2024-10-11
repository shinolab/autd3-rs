use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{format_ident, quote, ToTokens};
use syn::{Field, Ident, Meta};

fn get_fields<I>(input: &syn::DeriveInput, ident: &I) -> Vec<Field>
where
    I: ?Sized,
    Ident: PartialEq<I>,
{
    if let syn::Data::Struct(syn::DataStruct { fields, .. }) = input.data.clone() {
        fields
            .iter()
            .filter_map(|field| {
                if field.attrs.iter().any(|attr| match &attr.meta {
                    Meta::Path(path) if path.is_ident(ident) => true,
                    Meta::List(list) if list.path.is_ident(ident) => true,
                    _ => false,
                }) {
                    Some(field.clone())
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    }
}

fn has_attr(field: &Field, ident: &str) -> bool {
    field.attrs.iter().any(|attr| match &attr.meta {
        Meta::List(list) => list.tokens.clone().into_iter().any(|token| match token {
            TokenTree::Ident(i) => i == ident,
            _ => false,
        }),
        _ => false,
    })
}

fn impl_getter(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let generics = &input.generics;

    let getter_fileds = get_fields(input, "get");

    let getters = getter_fileds.iter().filter_map(|field| {
        let ty = &field.ty;
        field.ident.as_ref().map(|ident| {
            if let syn::Type::Path(path) = ty {
                let path = path.path.to_token_stream().to_string();
                {
                    let re = regex::Regex::new(r"LazyCell < RefCell < Vec < (?<inner>\w+) > > >")
                        .unwrap();
                    if let Some(caps) = re.captures(&path) {
                        let inner = format_ident!("{}", &caps["inner"]);
                        let mut_name = format_ident!("{}_mut", ident);
                        return quote! {
                            #[must_use]
                            pub fn #ident(&self) -> std::cell::Ref<'_, Vec<#inner>> {
                                self.#ident.borrow()
                            }
                            #[must_use]
                            pub fn #mut_name(&self) -> std::cell::RefMut<'_, Vec<#inner>> {
                                self.#ident.borrow_mut()
                            }
                        };
                    };
                }
                {
                    let re = regex::Regex::new(r"Vec < (?<inner>\w+) >").unwrap();
                    if let Some(caps) = re.captures(&path) {
                        let inner = format_ident!("{}", &caps["inner"]);
                        return quote! {
                            #[must_use]
                            pub fn #ident(&self) -> &[#inner] {
                                &self.#ident
                            }
                        };
                    };
                }
            };
            if has_attr(field, "ref") {
                if has_attr(field, "ref_mut") {
                    let mut_name = format_ident!("{}_mut", ident);
                    quote! {
                        #[must_use]
                        pub const fn #ident(&self) -> &#ty {
                            &self.#ident
                        }
                        #[must_use]
                        pub fn #mut_name(&mut self) -> &mut #ty {
                            &mut self.#ident
                        }
                    }
                } else {
                    quote! {
                        #[must_use]
                        pub const fn #ident(&self) -> &#ty {
                            &self.#ident
                        }
                    }
                }
            } else {
                quote! {
                    #[must_use]
                    pub const fn #ident(&self) -> #ty {
                        self.#ident
                    }
                }
            }
        })
    });

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let const_params = generics.const_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl <#(#linetimes,)* #(#type_params,)* #(#const_params,)*> #name #ty_generics #where_clause {
           #(#getters)*
        }
    }
}

fn impl_setter(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let generics = &input.generics;

    let setter_fileds = get_fields(input, "set");

    let setters = setter_fileds.iter().filter_map(|field| {
        let ty = &field.ty;
        field.ident.as_ref().map(|ident| {
            let name = format_ident!("with_{}", ident);
            if has_attr(field, "into") {
                quote! {
                    #[allow(clippy::needless_update)]
                    #[must_use]
                    pub fn #name(mut self, #ident: impl Into<#ty>) -> Self {
                        self.#ident = #ident.into();
                        self
                    }
                }
            } else {
                let const_attr = if has_attr(field, "no_const") {
                    quote! {}
                } else {
                    quote! { const }
                };
                quote! {
                    #[allow(clippy::needless_update)]
                    #[must_use]
                    pub #const_attr fn #name(mut self, #ident: #ty) -> Self {
                        self.#ident = #ident;
                        self
                    }
                }
            }
        })
    });

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let const_params = generics.const_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl <#(#linetimes,)* #(#type_params,)* #(#const_params,)*> #name #ty_generics #where_clause {
           #(#setters)*
        }
    }
}

pub(crate) fn impl_builder_macro(input: syn::DeriveInput) -> TokenStream {
    let getters = impl_getter(&input);
    let setters = impl_setter(&input);

    let generator = quote! {
        #getters

        #setters
    };
    generator.into()
}
