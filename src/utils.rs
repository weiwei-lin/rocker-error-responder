use attrs::Attrs;
use proc_macro2::Ident;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Data, DeriveInput, Error, Generics, Result, Token,
};

use crate::attrs;

struct Wrap<T>(pub T);

impl<T: Parse> Parse for Wrap<Punctuated<T, Token![,]>> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse_terminated(T::parse)?))
    }
}

pub struct Item {
    pub ident: Ident,
    pub generics: Generics,
    pub data: ItemData,
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let repr: DeriveInput = input.parse()?;
        let attrs = attrs::get(repr.attrs.as_slice())?;

        let data = match repr.data {
            Data::Enum(data) => {
                if let Some(delegate) = attrs.delegate {
                    return Err(Error::new_spanned(
                        delegate.kw,
                        "delegate cannot be applied on enums",
                    ));
                }
                let variants = data
                    .variants
                    .iter()
                    .map(|v| {
                        let mut variant_attrs = attrs::get(v.attrs.as_slice())?;
                        if variant_attrs.delegate.is_none() {
                            variant_attrs.code = variant_attrs.code.or_else(|| attrs.code.clone());
                            if variant_attrs.code.is_none() {
                                return Err(Error::new_spanned(
                                    v,
                                    "code or delegate must be specified",
                                ));
                            }
                        }

                        Ok(Variant {
                            repr: v.clone(),
                            attrs: variant_attrs,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                ItemData::Enum(ItemDataEnum { variants })
            }
            Data::Struct(data) => {
                if attrs.delegate.is_none() && attrs.code.is_none() {
                    return Err(Error::new_spanned(
                        repr.ident.clone(),
                        "code or delegate must be specified",
                    ));
                }
                ItemData::Struct(ItemDataStruct { attrs, repr: data })
            }
            Data::Union(_) => {
                if attrs.delegate.is_none() && attrs.code.is_none() {
                    return Err(Error::new_spanned(
                        repr.ident.clone(),
                        "code must be specified",
                    ));
                }
                ItemData::Union(ItemDataUnion { attrs })
            }
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
    pub repr: syn::DataStruct,
    pub attrs: Attrs,
}

pub struct ItemDataUnion {
    pub attrs: Attrs,
}

pub struct Variant {
    pub repr: syn::Variant,
    pub attrs: Attrs,
}
