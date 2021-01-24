use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Fields};
use utils::{Item, ItemData};

mod utils;

#[proc_macro_derive(GetStatus, attributes(response))]
pub fn derive_get_status_code(input: TokenStream) -> TokenStream {
    let input: Item = parse_macro_input!(input);

    let (impl_generics, ty_generics, where_clauses) = input.generics.split_for_impl();
    let ident = input.ident;

    let fn_impl = match input.data {
        ItemData::Enum(data) => {
            let arms = data.variants.iter().map(|v| {
                let code = v.code;
                let ident = &v.repr.ident;
                match v.repr.fields {
                    Fields::Named(..) => {
                        quote! {
                            Self::#ident{..} => #code,
                        }
                    }
                    Fields::Unnamed(..) => {
                        quote! {
                            Self::#ident(..) => #code,
                        }
                    }
                    Fields::Unit => {
                        quote! {
                            Self::#ident => #code,
                        }
                    }
                }
            });
            quote! {
                let code = match self {
                    #(#arms)*
                };
                ::rocket::http::Status::from_code(code).unwrap()
            }
        }
        ItemData::Struct(data) => {
            let code = data.code;
            quote! {
                ::rocket::http::Status::from_code(#code).unwrap()
            }
        }
        ItemData::Union(data) => {
            let code = data.code;
            quote! {
                ::rocket::http::Status::from_code(#code).unwrap()
            }
        }
    };
    (quote! {
        impl#impl_generics ::rocket_simple_responder::GetStatus for #ident#ty_generics
        #where_clauses
        {
            fn get_status(&self) -> ::rocket::http::Status {
                #fn_impl
            }
        }
    })
    .into()
}

#[proc_macro_derive(SimpleResponder, attributes(response))]
pub fn derive_maskable(input: TokenStream) -> TokenStream {
    let input: Item = parse_macro_input!(input);

    let mut impl_generics = input.generics.clone();
    impl_generics.params.push(parse_quote!('_r));
    impl_generics.params.push(parse_quote!('_o: '_r));
    let (_, ty_generics, where_clauses) = input.generics.split_for_impl();

    let ident = input.ident;

    (quote! {
        impl#impl_generics ::rocket::response::Responder<'_r, '_o> for #ident#ty_generics
        #where_clauses
        {
            fn respond_to(self, _request: &'_r ::rocket::Request<'_>) -> ::rocket::response::Result<'_o> {
                let status = ::rocket_simple_responder::GetStatus::get_status(&self);
                let msg = ::std::string::ToString::to_string(&self);
                Ok(::rocket::Response::build()
                    .status(status)
                    .header(::rocket::http::ContentType::Plain)
                    .sized_body(msg.len(), ::std::io::Cursor::new(msg))
                    .finalize())
            }
        }
    })
    .into()
}
