use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Fields};
use utils::{Item, ItemData};

mod attrs;
mod utils;

#[proc_macro_derive(SimpleResponder, attributes(response))]
pub fn derive_maskable(input: TokenStream) -> TokenStream {
    let input: Item = parse_macro_input!(input);

    let mut impl_generics = input.generics.clone();
    impl_generics.params.push(parse_quote!('_r));
    impl_generics.params.push(parse_quote!('_o: '_r));
    let (_, ty_generics, where_clauses) = input.generics.split_for_impl();

    let ident = input.ident;

    let responder_impl = match input.data {
        ItemData::Enum(data) => {
            let arms = data.variants.iter().map(|v| {
                let ident = &v.repr.ident;
                if let Some(delegate) = &v.attrs.delegate {
                    let expr = &delegate.value;
                    let patterns = fields_pat(&v.repr.fields);
                    quote! {
                        Self::#ident#patterns => ::rocket::response::Responder::respond_to(#expr, request),
                    }
                } else if let Some(code) = &v.attrs.code {
                    let code = code.code;
                    let patterns = match &v.repr.fields {
                        Fields::Named(..) => quote!{(..)},
                        Fields::Unnamed(..) => quote!{(..)},
                        Fields::Unit => quote!{},
                    };
                    quote! {
                        Self::#ident#patterns => {
                            let msg = ::std::string::ToString::to_string(&self);
                            Ok(::rocket::Response::build()
                                .status(::rocket::http::Status::from_code(#code).unwrap())
                                .header(::rocket::http::ContentType::Plain)
                                .sized_body(msg.len(), ::std::io::Cursor::new(msg))
                                .finalize())
                        }
                    }
                } else {
                    panic!("should have one of delegate or code");
                }
            });
            quote! { match self { #(#arms)* } }
        }
        ItemData::Struct(data) => {
            if let Some(delegate) = data.attrs.delegate {
                let expr = delegate.value;
                let patterns = fields_pat(&data.repr.fields);
                quote! {{
                    let Self#patterns = self;
                    ::rocket::response::Responder::respond_to(#expr, request)
                }}
            } else if let Some(code) = data.attrs.code {
                let code = code.code;
                quote! {{
                    let msg = ::std::string::ToString::to_string(&self);
                    Ok(::rocket::Response::build()
                        .status(::rocket::http::Status::from_code(#code).unwrap())
                        .header(::rocket::http::ContentType::Plain)
                        .sized_body(msg.len(), ::std::io::Cursor::new(msg))
                        .finalize())
                }}
            } else {
                panic!("should have one of delegate or code");
            }
        }
        ItemData::Union(data) => {
            let code = data.attrs.code.expect("should have code").code;
            quote! {{
                let msg = ::std::string::ToString::to_string(&self);
                Ok(::rocket::Response::build()
                    .status(::rocket::http::Status::from_code(#code).unwrap())
                    .header(::rocket::http::ContentType::Plain)
                    .sized_body(msg.len(), ::std::io::Cursor::new(msg))
                    .finalize())
            }}
        }
    };

    (quote! {
        impl#impl_generics ::rocket::response::Responder<'_r, '_o> for #ident#ty_generics
        #where_clauses
        {
            fn respond_to(self, request: &'_r ::rocket::Request<'_>) -> ::rocket::response::Result<'_o> {
                #responder_impl
            }
        }
    })
    .into()
}

fn fields_pat(fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(fields) => {
            let fields = &fields.named;
            quote! {{ #fields }}
        }
        Fields::Unnamed(fields) => {
            let fields = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _f)| format_ident!("_{}", i));
            quote! {(#(#fields)*,)}
        }
        Fields::Unit => {
            quote! {}
        }
    }
}
