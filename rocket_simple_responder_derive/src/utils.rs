use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use rocket::http::Status;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Data, DeriveInput, Generics, Lit, Meta, MetaNameValue, NestedMeta, Token,
};

struct Wrap<T>(pub T);

impl<T: Parse> Parse for Wrap<Punctuated<T, Token![,]>> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse_terminated(T::parse)?))
    }
}

pub struct Item {
    pub ident: Ident,
    pub generics: Generics,
    pub data: ItemData,
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let repr: DeriveInput = input.parse()?;
        let code = get_code(repr.attrs.as_slice())?;

        let data = match repr.data {
            Data::Enum(data) => {
                let variants = data
                    .variants
                    .iter()
                    .map(|v| {
                        let variant_code =
                            get_code(v.attrs.as_slice())?.or(code).ok_or_else(|| {
                                syn::Error::new_spanned(v, "status code must be specified")
                            })?;

                        Ok(Variant {
                            repr: v.clone(),
                            code: variant_code,
                        })
                    })
                    .collect::<syn::Result<Vec<_>>>()?;
                ItemData::Enum(ItemDataEnum { variants })
            }
            Data::Struct(_) => ItemData::Struct(ItemDataStruct {
                code: code.ok_or_else(|| {
                    syn::Error::new_spanned(repr.ident.clone(), "status code must be specified")
                })?,
            }),
            Data::Union(_) => ItemData::Union(ItemDataUnion {
                code: code.ok_or_else(|| {
                    syn::Error::new_spanned(repr.ident.clone(), "status code must be specified")
                })?,
            }),
        };

        Ok(Self {
            ident: repr.ident,
            generics: repr.generics,
            data,
        })
    }
}

pub enum ItemData {
    Enum(ItemDataEnum),
    Struct(ItemDataStruct),
    Union(ItemDataUnion),
}
pub struct ItemDataEnum {
    pub variants: Vec<Variant>,
}

pub struct ItemDataStruct {
    pub code: u16,
}

pub struct ItemDataUnion {
    pub code: u16,
}

pub struct Variant {
    pub repr: syn::Variant,
    pub code: u16,
}

pub struct CodeAttributeArg {
    origin: MetaNameValue,
    code: u16,
}

impl ToTokens for CodeAttributeArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.origin.to_tokens(tokens);
    }
}

impl Parse for CodeAttributeArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let meta: NestedMeta = input.parse()?;
        match meta {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                path,
                eq_token,
                lit: Lit::Int(lit),
            })) if path.is_ident("code") => Ok(Self {
                code: lit.base10_parse()?,
                origin: MetaNameValue {
                    path,
                    eq_token,
                    lit: Lit::Int(lit),
                },
            }),
            _ => Err(syn::Error::new_spanned(meta, "invalid meta")),
        }
    }
}

pub fn get_code(attrs: &[Attribute]) -> syn::Result<Option<u16>> {
    let mut code_attr_iter = attrs
        .iter()
        .filter(|attr| attr.path.is_ident("simple_responder"))
        .map(|attr| attr.parse_args())
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|attrs: Wrap<Punctuated<CodeAttributeArg, Token![,]>>| attrs.0);
    let code = code_attr_iter
        .next()
        .map::<syn::Result<_>, _>(|code_attr| {
            let code = code_attr.code;
            Status::from_code(code)
                .ok_or_else(|| syn::Error::new_spanned(code_attr, "invalid status code"))?;
            Ok(code)
        })
        .transpose()?;
    if let Some(code_attr) = code_attr_iter.next() {
        return Err(syn::Error::new_spanned(
            code_attr,
            "only one code attribute is allowed",
        ));
    }
    Ok(code)
}
