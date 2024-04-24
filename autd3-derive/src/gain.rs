use proc_macro::TokenStream;
use quote::quote;
use syn::WhereClause;

fn to_gain_where(where_clause: Option<&WhereClause>) -> proc_macro2::TokenStream {
    match where_clause {
        Some(where_clause) => {
            let where_predicate_punctuated_list = where_clause
                .predicates
                .iter()
                .map(|where_predicate| match where_predicate {
                    syn::WherePredicate::Type(_) => {
                        quote! { #where_predicate }
                    }
                    _ => quote! {},
                })
                .collect::<Vec<_>>();
            quote! { where GainOp<Self>: Operation, #(#where_predicate_punctuated_list),* }
        }
        None => {
            quote! { where GainOp<Self>: Operation }
        }
    }
}

pub(crate) fn impl_gain_macro(ast: syn::DeriveInput) -> TokenStream {
    let attrs = &ast.attrs;
    let name = &ast.ident;
    let generics = &ast.generics;

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let where_clause = to_gain_where(where_clause);
    let cache = if attrs
        .iter()
        .any(|attr| attr.path().is_ident("no_gain_cache"))
    {
        quote! {}
    } else {
        quote! {
            impl <#(#linetimes,)* #(#type_params,)*> IntoGainCache<Self> for #name #ty_generics #where_clause {
                fn with_cache(self) -> GainCache<Self> {
                    GainCache::new(self)
                }
            }
        }
    };

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let where_clause = to_gain_where(where_clause);
    let transform = if attrs
        .iter()
        .any(|attr| attr.path().is_ident("no_gain_transform"))
    {
        quote! {}
    } else {
        quote! {
            impl <#(#linetimes,)* #(#type_params,)*> IntoGainTransform<Self> for #name #ty_generics #where_clause {
                fn with_transform<GainTransformF: Fn(&Device, &Transducer, Drive) -> Drive>(self, f: GainTransformF) -> GainTransform<Self, GainTransformF> {
                    GainTransform::new(self, f)
                }
            }
        }
    };

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let where_clause = to_gain_where(where_clause);
    let datagram = quote! {
        impl <#(#linetimes,)* #(#type_params,)*> DatagramS for #name #ty_generics #where_clause
        {
            type O1 = GainOp<Self>;
            type O2 = NullOp;

            fn operation_with_segment(self, segment: Segment, transition_mode: Option<TransitionMode>) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
                Ok((Self::O1::new(segment, transition_mode.is_some(), self), Self::O2::default()))
            }
        }
    };

    let gen = quote! {
        #datagram

        #cache

        #transform
    };
    gen.into()
}
