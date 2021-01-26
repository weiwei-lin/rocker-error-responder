use attrs::{FieldsAttrs, TypeAttrs};
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
        let ty_attrs = TypeAttrs::new(repr.attrs.as_slice())?;

        let data = match repr.data {
            Data::Enum(data) => {
                let variants = data
                    .variants
                    .iter()
                    .map(|v| {
                        let mut variant_attrs = TypeAttrs::new(v.attrs.as_slice())?;
                        let fields_attrs = FieldsAttrs::new(&v.fields)?;
                        if fields_attrs.delegate.is_some() && variant_attrs.code.is_some() {
                            return Err(Error::new_spanned(
                                fields_attrs.delegate.unwrap().kw,
                                "can't specify both code and delegate",
                            ));
                        }
                        variant_attrs.code = variant_attrs.code.or_else(|| ty_attrs.code.clone());
                        if fields_attrs.delegate.is_none() && variant_attrs.code.is_none() {
                            return Err(Error::new_spanned(
                                v.ident.clone(),
                                "code or delegate must be specified",
                            ));
                        }
                        Ok(Variant {
                            repr: v.clone(),
                            variant_attrs,
                            fields_attrs,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                ItemData::Enum(ItemDataEnum { variants })
            }
            Data::Struct(data) => {
                let fields_attrs = FieldsAttrs::new(&data.fields)?;
                if fields_attrs.delegate.is_some() && ty_attrs.code.is_some() {
                    return Err(Error::new_spanned(
                        fields_attrs.delegate.unwrap().kw,
                        "can't specify both code and delegate",
                    ));
                }
                if fields_attrs.delegate.is_none() && ty_attrs.code.is_none() {
                    return Err(Error::new_spanned(
                        repr.ident.clone(),
                        "code or delegate must be specified",
                    ));
                }
                ItemData::Struct(ItemDataStruct {
                    ty_attrs,
                    fields_attrs,
                    repr: data,
                })
            }
            Data::Union(data) => {
                if ty_attrs.code.is_none() {
                    return Err(Error::new_spanned(
                        repr.ident.clone(),
                        "code must be specified",
                    ));
                }
                let fields_attrs = FieldsAttrs::new(&data.fields.into())?;
                if let Some(delegate) = fields_attrs.delegate {
                    return Err(Error::new_spanned(
                        delegate.kw,
                        "can't use delegate on union type",
                    ));
                }
                ItemData::Union(ItemDataUnion { ty_attrs })
            }
        };

        Ok(Self {
            ident: repr.ident,
            generics: repr.generics,
            data,
        })
    }
}

#[allow(clippy::large_enum_variant)]
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
    pub ty_attrs: TypeAttrs,
    pub fields_attrs: FieldsAttrs,
}

pub struct ItemDataUnion {
    pub ty_attrs: TypeAttrs,
}

pub struct Variant {
    pub repr: syn::Variant,
    pub variant_attrs: TypeAttrs,
    pub fields_attrs: FieldsAttrs,
}
