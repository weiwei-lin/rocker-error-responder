use proc_macro2::Ident;
use quote::format_ident;
use rocket::http::Status;
use syn::{
    custom_keyword,
    parse::{Parse, ParseStream},
    Attribute, Error, Fields, LitInt, Result, Token, Type,
};

custom_keyword!(code);
custom_keyword!(delegate);

pub struct TypeAttrs {
    pub code: Option<CodeArg>,
}

impl TypeAttrs {
    pub fn new(input: &[Attribute]) -> Result<Self> {
        let mut ret = Self { code: None };
        for attr in input.iter().filter(|a| a.path.is_ident("response")) {
            ret.parse_attrs(attr)?;
        }
        Ok(ret)
    }

    fn parse_attrs(&mut self, attr: &Attribute) -> Result<()> {
        attr.parse_args_with(|input: ParseStream| {
            loop {
                let lookahead = input.lookahead1();
                if lookahead.peek(code) {
                    if self.code.is_some() {
                        return Err(Error::new_spanned(
                            input.parse::<code>().unwrap(),
                            "duplicate code argument",
                        ));
                    }
                    self.code = Some(input.parse()?);
                } else {
                    return Err(lookahead.error());
                }
                if input.parse::<Option<Token![,]>>()?.is_none() {
                    break;
                }
            }
            Ok(())
        })
    }
}

#[derive(Clone)]
pub struct CodeArg {
    pub kw: code,
    pub eq_token: Token![=],
    pub code: u16,
}

impl Parse for CodeArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let kw = input.parse()?;
        let eq_token = input.parse()?;
        let code_lit: LitInt = input.parse()?;
        let code = code_lit.base10_parse()?;
        Status::from_code(code)
            .ok_or_else(|| Error::new_spanned(code_lit, "invalid status code"))?;
        Ok(Self { kw, eq_token, code })
    }
}

pub struct FieldsAttrs {
    pub delegate: Option<Delegate>,
}

pub struct Delegate {
    pub kw: delegate,
    pub ident: Ident,
    pub ty: Type,
}

impl FieldsAttrs {
    pub fn new(fields: &Fields) -> Result<Self> {
        let mut ret = Self { delegate: None };
        for (i, field) in fields.iter().enumerate() {
            for attr in field.attrs.iter().filter(|a| a.path.is_ident("response")) {
                if let Some(kw) = ret.parse_attrs(attr)? {
                    ret.delegate = Some(Delegate {
                        kw,
                        ident: field
                            .ident
                            .clone()
                            .unwrap_or_else(|| format_ident!("_{}", i)),
                        ty: field.ty.clone(),
                    });
                }
            }
        }
        Ok(ret)
    }

    fn parse_attrs(&self, attr: &Attribute) -> Result<Option<delegate>> {
        attr.parse_args_with(|input: ParseStream| {
            let mut ret = self.delegate.as_ref().map(|d| d.kw);
            loop {
                let lookahead = input.lookahead1();
                if lookahead.peek(delegate) {
                    let kw = input.parse::<delegate>()?;
                    if ret.is_some() {
                        return Err(Error::new_spanned(kw, "duplicate delegate tag"));
                    }
                    ret = Some(kw);
                } else {
                    return Err(lookahead.error());
                }
                if input.parse::<Option<Token![,]>>()?.is_none() {
                    break;
                }
            }
            Ok(ret)
        })
    }
}
