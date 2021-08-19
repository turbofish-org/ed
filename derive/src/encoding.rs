use quote::quote;
use syn::*;
use syn::punctuated::Punctuated;
use proc_macro2::TokenStream;

// TODO: use correct spans so errors are shown on fields

pub fn derive_encode(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    // let name = &item.ident;

    // let field_names: Vec<_> = struct_fields(&item).map(|field| &field.ident).collect();

    // let mut field_names_minus_last: Vec<_> =
    //     struct_fields(&item).map(|field| &field.ident).collect();
    // field_names_minus_last.pop();

    // let output = quote! {
    //     impl ed::Encode for #name {
    //         #[inline]
    //         fn encode_into<W: std::io::Write>(&self, mut dest: &mut W) -> ed::Result<()> {
    //             fn assert_trait_bounds<T: ed::Encode + ed::Terminated>(_: &T) {}
    //             #(assert_trait_bounds(&self.#field_names_minus_last);)*

    //             #(self.#field_names.encode_into(&mut dest)?;)*
    //         }

    //         #[inline]
    //         fn encoding_length(&self) -> ed::Result<usize> {
    //             Ok(
                    
    //             )
    //         }
    //     }
    // };

    quote!().into()
}

pub fn derive_decode(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    let output = match item.data {
        Data::Struct(data) => struct_decode(item.ident, data),
        Data::Enum(data) => enum_decode(item.ident, data),
        Data::Union(_) => unimplemented!("Not implemented for unions"),
    };

    output.into()
}

fn struct_decode(name: Ident, data: DataStruct) -> TokenStream {
    let decode = fields_decode(&data.fields, None);
    let decode_into = fields_decode_into(&data.fields, None);

    quote! {
        impl ed::Decode for #name {
            #[inline]
            fn decode<R: std::io::Read>(mut input: R) -> ed::Result<Self> {
                Ok(#decode)
            }

            #[inline]
            fn decode_into<R: std::io::Read>(&mut self, mut input: R) -> ed::Result<()> {
                #decode_into
                Ok(())
            }
        }
    }
}

fn enum_decode(name: Ident, data: DataEnum) -> TokenStream {
    let mut arms = data.variants.iter().enumerate().map(|(i, v)| {
        let i = i as u8;
        let arm = fields_decode(&v.fields, Some(v.ident.clone()));
        quote!(#i => { #arm })
    });

    quote! {
        impl ::ed::Decode for #name {
            #[inline]
            fn decode<R: std::io::Read>(mut input: R) -> ::ed::Result<Self> {
                let mut variant = [0; 1];
                input.read_exact(&mut variant[..])?;
                let variant = variant[0];
    
                Ok(match variant {
                    #(#arms),*
                    n => return Err(::ed::Error::UnexpectedByte(n)),
                })
            }

            // TODO: decode_into
        }
    }
}

fn iter_fields(fields: &Fields) -> Box<dyn Iterator<Item=Field>> {
    match fields.clone() {
        Fields::Named(fields) => Box::new(fields.named.into_iter()),
        Fields::Unnamed(fields) => Box::new(fields.unnamed.into_iter()),
        Fields::Unit => Box::new(vec![].into_iter()),
    }
}

fn fields_encode_into(fields: &Fields) -> TokenStream {
    let mut field_names_minus_last: Vec<_> = iter_fields(fields)
        .map(|field| field.ident)
        .collect();
    field_names_minus_last.pop();

    let mut field_names = iter_fields(fields).map(|field| field.ident);

    quote! {
        fn assert_trait_bounds<T: ed::Encode + ed::Terminated>(_: &T) {}
        #(assert_trait_bounds(&self.#field_names_minus_last);)*

        #(self.#field_names.encode_into(&mut dest)?;)*
    }
}

fn fields_encoding_length(fields: &Fields) -> TokenStream {
    let mut field_names = iter_fields(fields).map(|field| field.ident);

    quote! {
        0 #( + self.#field_names.encoding_length()?)*
    }
}

fn fields_decode(fields: &Fields, variant_name: Option<Ident>) -> TokenStream {
    let mut field_names = iter_fields(fields)
        .enumerate()
        .map(|(i, field)| match &field.ident {
            Some(ident) => quote!(#ident),
            None => quote!(#i),
        });
    
    let item_name = match variant_name {
        Some(name) => quote!(Self::#name),
        None => quote!(Self),
    };

    quote! {
        #item_name {
            #(
                #field_names: ::ed::Decode::decode(&mut input)?,
            )*
        }
    }
}

fn fields_decode_into(fields: &Fields, parent: Option<TokenStream>) -> TokenStream {
    let mut field_names = iter_fields(fields)
        .enumerate()
        .map(|(i, field)| match &field.ident {
            Some(ident) => quote!(#ident),
            None => quote!(#i),
        });
    let parent = parent.unwrap_or(quote!(self));

    quote! {
        #(
            #parent.#field_names.decode_into(&mut input)?;
        )*
    }
}
