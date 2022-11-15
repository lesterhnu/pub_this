use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item, ItemStruct, Result};

/// this macro can do nothing but make your structs' fields be public
/// 这个宏只能让你的struct的字段变为pub
#[proc_macro_attribute]
pub fn pub_this(_args: TokenStream, input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as Item);

    match do_expand(&st) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn do_expand(st: &Item) -> Result<proc_macro2::TokenStream> {
    match st {
        Item::Struct(e) => {
            let vis = &e.vis;
            let generics = &e.generics;
            let struc_ident = &e.ident;
            let fields = get_struct_fields(e)?;
            let idents: Vec<_> = fields.into_iter().map(|f| &f.ident).collect();
            let types: Vec<_> = fields
                .into_iter()
                .map(|f| {
                    if let Some(inner_type) = get_optionanl_inner_type(&f.ty) {
                        quote!(
                            Option<#inner_type>
                        )
                    } else {
                        let origin_type = &f.ty;
                        quote!(
                            #origin_type
                        )
                    }
                })
                .collect();

            let ret = quote!(
                #vis struct #struc_ident #generics{
                    #(pub #idents: #types),*
                }
            );
            Ok(ret)
        }
        _ => Err(syn::Error::new_spanned(
            st,
            "Must Define On struct! 只能用在struct上".to_string(),
        )),
    }
}

type StructFields = syn::punctuated::Punctuated<syn::Field, syn::Token![,]>;
fn get_struct_fields(st: &ItemStruct) -> Result<&StructFields> {
    if let syn::ItemStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    } = st
    {
        return Ok(named);
    };
    Err(syn::Error::new_spanned(
        st,
        "Must Define On struct,".to_string(),
    ))
}

fn get_optionanl_inner_type(t: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { segments, .. },
        ..
    }) = t
    {
        if let Some(seg) = segments.last() {
            if seg.ident.to_string() == "Option" {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) = &seg.arguments
                {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.first() {
                        return Some(inner_type);
                    }
                }
            }
        }
    }
    None
}
