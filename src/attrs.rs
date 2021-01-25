use std::iter::FromIterator;

use proc_macro2::{TokenStream, TokenTree};
use quote::format_ident;
use rocket::http::Status;
use syn::{
    custom_keyword,
    parse::{Parse, ParseStream},
    Attribute, Error, Ident, Index, LitInt, Result, Token,
};

custom_keyword!(code);
custom_keyword!(delegate);

pub struct Attrs {
    pub code: Option<CodeArg>,
    pub delegate: Option<DelegateArg>,
}

pub fn get(input: &[Attribute]) -> Result<Attrs> {
    let mut attrs = Attrs {
        code: None,
        delegate: None,
    };
    for attr in input.iter().filter(|a| a.path.is_ident("response")) {
        parse_attrs(&mut attrs, attr)?;
    }
    Ok(attrs)
}

fn parse_attrs(args: &mut Attrs, attr: &Attribute) -> Result<()> {
    attr.parse_args_with(|input: ParseStream| {
        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(code) {
                if args.code.is_some() {
                    return Err(Error::new_spanned(
                        input.parse::<code>().unwrap(),
                        "duplicate code argument",
                    ));
                }
                if args.delegate.is_some() {
                    return Err(Error::new_spanned(
                        input.parse::<delegate>().unwrap(),
                        "cannot specified both delegate and code",
                    ));
                }
                args.code = Some(input.parse()?);
            } else if lookahead.peek(delegate) {
                if args.delegate.is_some() {
                    return Err(Error::new_spanned(
                        input.parse::<delegate>().unwrap(),
                        "duplicate delegate argument",
                    ));
                }
                if args.code.is_some() {
                    return Err(Error::new_spanned(
                        input.parse::<code>().unwrap(),
                        "cannot specified both delegate and code",
                    ));
                }
                args.delegate = Some(input.parse()?);
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

pub struct DelegateArg {
    pub kw: delegate,
    pub eq_token: Token![=],
    pub value: TokenStream,
}

impl Parse for DelegateArg {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            kw: input.parse()?,
            eq_token: input.parse()?,
            value: parse_delegate_arg_value(input)?,
        })
    }
}

fn parse_delegate_arg_value(input: ParseStream) -> Result<TokenStream> {
    let mut tokens = Vec::new();
    input.parse::<Token![.]>()?;
    let lookahead = input.lookahead1();
    if lookahead.peek(Ident) {
        tokens.push(TokenTree::Ident(input.parse()?))
    } else if lookahead.peek(LitInt) {
        let int: Index = input.parse()?;
        let ident = format_ident!("_{}", int.index, span = int.span);
        tokens.push(TokenTree::Ident(ident));
    } else {
        return Err(lookahead.error());
    }
    while !input.peek(Token![,]) && !input.is_empty() {
        input.parse::<Token![.]>()?;
        if lookahead.peek(Ident) {
            tokens.push(TokenTree::Ident(input.parse()?))
        } else if lookahead.peek(LitInt) {
            tokens.push(TokenTree::Literal(input.parse()?));
        } else {
            return Err(lookahead.error());
        }
    }
    Ok(TokenStream::from_iter(tokens))
}
