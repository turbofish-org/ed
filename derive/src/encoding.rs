use quote::quote;
use syn::*;

// TODO: use correct spans so errors are shown on fields

pub fn derive_encode(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    let name = &item.ident;

    let field_names: Vec<_> = struct_fields(&item).map(|field| &field.ident).collect();

    let mut field_names_minus_last: Vec<_> =
        struct_fields(&item).map(|field| &field.ident).collect();
    field_names_minus_last.pop();

    let output = quote! {
        impl ed::Encode for #name {
            fn encode_into<W: std::io::Write>(&self, mut dest: &mut W) -> ed::Result<()> {
                fn assert_trait_bounds<T: ed::Encode + ed::Terminated>(_: &T) {}
                #(assert_trait_bounds(&self.#field_names_minus_last);)*

                #(self.#field_names.encode_into(&mut dest)?;)*

                Ok(())
            }

            fn encoding_length(&self) -> ed::Result<usize> {
                Ok(
                    0 #( + self.#field_names.encoding_length()?)*
                )
            }
        }
    };

    output.into()
}

pub fn derive_decode(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    let name = &item.ident;

    let field_names: Vec<_> = struct_fields(&item).map(|field| &field.ident).collect();

    let output = quote! {
        impl ed::Decode for #name {
            fn decode<R: std::io::Read>(mut input: R) -> ed::Result<Self> {
                Ok(Self {
                    #(
                        #field_names: ed::Decode::decode(&mut input)?,
                    )*
                })
            }

            fn decode_into<R: std::io::Read>(&mut self, mut input: R) -> ed::Result<()> {
                #(
                    self.#field_names.decode_into(&mut input)?;
                )*

                Ok(())
            }
        }
    };

    output.into()
}

fn struct_fields<'a>(
    item: &'a DeriveInput
) -> impl Iterator<Item=&'a Field> {
    let data = match item.data {
        Data::Struct(ref data) => data,
        _ => panic!("Currently only structs are supported")
    };
    match data.fields {
        Fields::Named(ref fields) => fields.named.iter(),
        Fields::Unnamed(ref fields) => fields.unnamed.iter(),
        Fields::Unit => panic!("Unit structs are not supported")
    }
}
