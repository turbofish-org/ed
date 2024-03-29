mod encoding;

#[proc_macro_derive(Encode, attributes(skip))]
pub fn encode(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    encoding::derive_encode(item)
}

#[proc_macro_derive(Decode, attributes(skip))]
pub fn decode(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    encoding::derive_decode(item)
}
