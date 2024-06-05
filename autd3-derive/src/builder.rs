use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::Meta;

fn impl_getter(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let generics = &input.generics;

    let getter_fileds =
        if let syn::Data::Struct(syn::DataStruct { fields, .. }) = input.data.clone() {
            fields
                .iter()
                .filter_map(|field| {
                    if field.attrs.iter().any(|attr| match &attr.meta {
                        Meta::Path(path) if path.is_ident("get") || path.is_ident("getset") => true,
                        Meta::List(list)
                            if list.path.is_ident("get") || list.path.is_ident("getset") =>
                        {
                            true
                        }
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
        };

    let getters = getter_fileds.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();

        let ty = &field.ty;
        match ty {
            syn::Type::Path(path) if path.path.is_ident("String") => quote! {
                pub fn #ident(&self) -> &str {
                    &self.#ident
                }
            },
            syn::Type::Path(path) if path.path.is_ident("Vector3") => quote! {
                pub const fn #ident(&self) -> &Vector3 {
                    &self.#ident
                }
            },
            syn::Type::Path(path) if path.path.is_ident("UnitQuaternion") => quote! {
                pub const fn #ident(&self) -> &UnitQuaternion {
                    &self.#ident
                }
            },
            syn::Type::Path(path) if path.path.is_ident("F") => quote! {
                pub const fn #ident(&self) -> &F {
                    &self.#ident
                }
            },
            syn::Type::Path(path) => {
                let re = regex::Regex::new(r"Vec < (?<inner>\w+) >").unwrap();
                if let Some(caps) = re.captures(&path.path.to_token_stream().to_string()) {
                    let inner = format_ident!("{}", &caps["inner"]);
                    quote! {
                        pub fn #ident(&self) -> &[#inner] {
                            &self.#ident
                        }
                    }
                } else {
                    quote! {
                        pub const fn #ident(&self) -> #ty {
                            self.#ident
                        }
                    }
                }
            }
            _ => quote! {
                pub const fn #ident(&self) -> #ty {
                    self.#ident
                }
            },
        }
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
    let attrs = &input.attrs;
    let name = &input.ident;
    let generics = &input.generics;

    let const_qua = if attrs.iter().any(|attr| attr.path().is_ident("no_const")) {
        quote! {}
    } else {
        quote! {const}
    };

    let setter_fileds =
        if let syn::Data::Struct(syn::DataStruct { fields, .. }) = input.data.clone() {
            fields
                .iter()
                .filter_map(|field| {
                    if field.attrs.iter().any(|attr| match &attr.meta {
                        Meta::Path(path) if path.is_ident("set") || path.is_ident("getset") => true,
                        Meta::List(list)
                            if list.path.is_ident("set") || list.path.is_ident("getset") =>
                        {
                            true
                        }
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
        };

    let setters = setter_fileds.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let attr = field
            .attrs
            .iter()
            .find(|attr| match &attr.meta {
                Meta::Path(path) if path.is_ident("set") || path.is_ident("getset") => true,
                Meta::List(list) if list.path.is_ident("set") || list.path.is_ident("getset") => {
                    true
                }
                _ => false,
            })
            .unwrap();
        if let Ok(syn::FnArg::Typed(typed)) = attr.parse_args::<syn::FnArg>() {
            let filed =
                syn::Ident::new(&typed.pat.to_token_stream().to_string(), Span::call_site());
            let name = format_ident!("with_{}", filed);
            let ty = typed.ty;
            quote! {
                pub fn #name(mut self, value: #ty) -> Self {
                    self.#ident.#filed = value;
                    self
                }
            }
        } else {
            let ty = &field.ty;
            let name = format_ident!("with_{}", ident);
            match ty {
                syn::Type::Path(path) if path.path.is_ident("EmitIntensity") => quote! {
                    pub fn #name(mut self, value: impl Into<EmitIntensity>) -> Self {
                        self.#ident = value.into();
                        self
                    }
                },
                syn::Type::Path(path) if path.path.is_ident("Phase") => quote! {
                    pub fn #name(mut self, value: impl Into<Phase>) -> Self {
                        self.#ident = value.into();
                        self
                    }
                },
                syn::Type::Path(path) if path.path.is_ident("UnitQuaternion") => quote! {
                    pub fn #name(mut self, value: impl Into<UnitQuaternion>) -> Self {
                        self.#ident = value.into();
                        self
                    }
                },
                syn::Type::Path(path) if path.path.is_ident("String") => quote! {
                    pub fn #name(mut self, value: impl Into<String>) -> Self {
                        self.#ident = value.into();
                        self
                    }
                },
                syn::Type::Path(path) if path.path.is_ident("IpAddr") => quote! {
                    pub fn #name(mut self, value: impl Into<IpAddr>) -> Self {
                        self.#ident = value.into();
                        self
                    }
                },
                _ => quote! {
                    #[allow(clippy::needless_update)]
                    pub #const_qua fn #name(mut self, #ident: #ty) -> Self {
                        self.#ident = #ident;
                        self
                    }
                },
            }
        }
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

    let gen = quote! {
        #getters

        #setters
    };
    gen.into()
}
