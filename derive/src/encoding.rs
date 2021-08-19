use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::*;

// TODO: use correct spans so errors are shown on fields

pub fn derive_encode(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    let output = match item.data.clone() {
        Data::Struct(data) => struct_encode(item, data),
        Data::Enum(data) => enum_encode(item, data),
        Data::Union(_) => unimplemented!("Not implemented for unions"),
    };

    println!("{}", &output);

    output.into()
}

fn struct_encode(item: DeriveInput, data: DataStruct) -> TokenStream {
    let name = &item.ident;

    let gen_bounds = gen_bounds_modified(
        &item.generics,
        vec![quote!(::ed::Encode), quote!(::ed::Terminated)],
    );
    let gen_params = gen_param_input(&item.generics);

    let encode_into = fields_encode_into(iter_field_names(&data.fields), Some(quote!(self)), false);
    let encoding_length =
        fields_encoding_length(iter_field_names(&data.fields), Some(quote!(self)));

    quote! {
        impl#gen_bounds ::ed::Encode for #name#gen_params {
            #[inline]
            fn encode_into<W: std::io::Write>(&self, mut dest: &mut W) -> ::ed::Result<()> {
                #encode_into

                Ok(())
            }

            #[inline]
            fn encoding_length(&self) -> ::ed::Result<usize> {
                Ok(#encoding_length)
            }
        }
    }
}

fn enum_encode(item: DeriveInput, data: DataEnum) -> TokenStream {
    let name = &item.ident;

    let gen_bounds = gen_bounds_modified(
        &item.generics,
        vec![quote!(::ed::Encode), quote!(::ed::Terminated)],
    );
    let gen_params = gen_param_input(&item.generics);

    let mut arms = data.variants.iter().enumerate().map(|(i, v)| {
        let i = i as u8;
        let ident = &v.ident;
        let destructure = variant_destructure(&v);
        let encode = fields_encode_into(iter_field_destructure(&v), None, true);
        quote!(Self::#ident #destructure => {
            dest.write_all(&[ #i ][..])?;
            #encode
        })
    });

    let encode_into = quote! {
        #[inline]
        fn encode_into<W: std::io::Write>(&self, mut dest: &mut W) -> ::ed::Result<()> {
            match self {
                #(#arms)*
            }

            Ok(())
        }
    };

    let mut arms = data.variants.iter().enumerate().map(|(i, v)| {
        let arm = fields_encoding_length(iter_field_destructure(&v), None);
        let ident = &v.ident;
        let destructure = variant_destructure(&v);
        quote!(Self::#ident #destructure => { #arm })
    });

    let encoding_length = quote! {
        #[inline]
        fn encoding_length(&self) -> ::ed::Result<usize> {
            Ok(1 + match self {
                #(#arms)*
            })
        }
    };

    quote! {
        impl#gen_bounds ::ed::Encode for #name#gen_params {
            #encode_into
            #encoding_length
        }
    }
}

pub fn derive_decode(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    let output = match item.data.clone() {
        Data::Struct(data) => struct_decode(item, data),
        Data::Enum(data) => enum_decode(item, data),
        Data::Union(_) => unimplemented!("Not implemented for unions"),
    };

    output.into()
}

fn struct_decode(item: DeriveInput, data: DataStruct) -> TokenStream {
    let name = &item.ident;

    let decode = fields_decode(&data.fields, None);
    let decode_into = fields_decode_into(&data.fields, None);

    let gen_bounds = gen_bounds_modified(
        &item.generics,
        vec![quote!(::ed::Decode), quote!(::ed::Terminated)],
    );
    let gen_params = gen_param_input(&item.generics);

    quote! {
        impl#gen_bounds ed::Decode for #name#gen_params {
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

fn enum_decode(item: DeriveInput, data: DataEnum) -> TokenStream {
    let name = &item.ident;

    let gen_bounds = gen_bounds_modified(
        &item.generics,
        vec![quote!(::ed::Decode), quote!(::ed::Terminated)],
    );
    let gen_params = gen_param_input(&item.generics);

    let mut arms = data.variants.iter().enumerate().map(|(i, v)| {
        let i = i as u8;
        let arm = fields_decode(&v.fields, Some(v.ident.clone()));
        quote!(#i => { #arm })
    });

    quote! {
        impl#gen_bounds ::ed::Decode for #name#gen_params {
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

fn iter_fields(fields: &Fields) -> Box<dyn Iterator<Item = Field>> {
    match fields.clone() {
        Fields::Named(fields) => Box::new(fields.named.into_iter()),
        Fields::Unnamed(fields) => Box::new(fields.unnamed.into_iter()),
        Fields::Unit => Box::new(vec![].into_iter()),
    }
}

fn iter_field_names(fields: &Fields) -> impl Iterator<Item = TokenStream> {
    iter_fields(fields)
        .enumerate()
        .map(|(i, field)| match field.ident {
            Some(ident) => quote!(#ident),
            None => {
                let i = Literal::usize_unsuffixed(i);
                quote!(#i)
            }
        })
}

fn iter_field_destructure(variant: &Variant) -> Box<dyn Iterator<Item = TokenStream>> {
    match variant.fields.clone() {
        Fields::Named(fields) => Box::new(fields.named.into_iter().map(|v| {
            let ident = v.ident;
            quote!(#ident)
        })),
        Fields::Unnamed(fields) => Box::new((0..variant.fields.len()).map(|i| {
            let ident = Ident::new(
                ("var".to_string() + i.to_string().as_str()).as_str(),
                Span::call_site(),
            );
            quote!(#ident)
        })),
        Fields::Unit => Box::new(vec![].into_iter()),
    }
}

fn variant_destructure(variant: &Variant) -> TokenStream {
    let names = iter_field_destructure(&variant);
    match &variant.fields {
        Fields::Named(fields) => quote!({ #(#names),* }),
        Fields::Unnamed(fields) => quote!(( #(#names),* )),
        Fields::Unit => quote!(),
    }
}

fn gen_bounds_modified(generics: &Generics, add: Vec<TokenStream>) -> TokenStream {
    let add: Vec<TypeParamBound> = add.iter().map(|add| parse_quote!(#add)).collect();

    let gen_bounds = generics.params.iter().cloned().map(|mut p| {
        if let GenericParam::Type(ref mut p) = p {
            for add in add.iter().cloned() {
                p.bounds.push(add);
            }
        }
        quote!(#p)
    });

    if gen_bounds.len() == 0 {
        quote!()
    } else {
        quote!(<#(#gen_bounds),*>)
    }
}

fn gen_param_input(generics: &Generics) -> TokenStream {
    let gen_params = generics.params.iter().map(|p| match p {
        GenericParam::Type(p) => {
            let ident = &p.ident;
            quote!(#ident)
        }
        GenericParam::Lifetime(p) => {
            let ident = &p.lifetime.ident;
            quote!(#ident)
        }
        GenericParam::Const(p) => {
            let ident = &p.ident;
            quote!(#ident)
        }
    });

    if gen_params.len() == 0 {
        quote!()
    } else {
        quote!(<#(#gen_params),*>)
    }
}

fn fields_encode_into(
    field_names: impl Iterator<Item = TokenStream>,
    parent: Option<TokenStream>,
    borrowed: bool,
) -> TokenStream {
    let mut field_names: Vec<_> = field_names.collect();
    let mut field_names_minus_last = field_names.clone();
    field_names_minus_last.pop();

    let assert_ampersand = if borrowed { quote!() } else { quote!(&) };

    let parent_dot = parent.as_ref().map(|_| quote!(.));

    quote! {
        fn assert_trait_bounds<T: ::ed::Encode + ::ed::Terminated>(_: &T) {}
        #(assert_trait_bounds(#assert_ampersand#parent#parent_dot#field_names_minus_last);)*

        #(#parent#parent_dot#field_names.encode_into(&mut dest)?;)*
    }
}

fn fields_encoding_length(
    field_names: impl Iterator<Item = TokenStream>,
    parent: Option<TokenStream>,
) -> TokenStream {
    let parent_dot = parent.as_ref().map(|_| quote!(.));

    quote! {
        0 #( + #parent#parent_dot#field_names.encoding_length()?)*
    }
}

fn fields_decode(fields: &Fields, variant_name: Option<Ident>) -> TokenStream {
    let mut field_names = iter_field_names(&fields);

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
    let mut field_names = iter_field_names(&fields);
    let parent = parent.unwrap_or(quote!(self));

    quote! {
        #(
            #parent.#field_names.decode_into(&mut input)?;
        )*
    }
}
