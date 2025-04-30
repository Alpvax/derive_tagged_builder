use darling::FromDeriveInput;
use proc_macro_error::proc_macro_error;
use quote::ToTokens;

mod builder;
mod options;
#[cfg(test)]
mod tests;

#[proc_macro_error]
#[proc_macro_derive(
    TaggedBuilder,
    attributes(
        builder,
        tagged_builder,
        builder_field_attr,
        tagged_builder_field_attr,
        builder_impl_attr,
        tagged_builder_impl_attr,
        builder_setter_attr,
        tagged_builder_setter_attr,
        builder_struct_attr,
        tagged_builder_struct_attr,
    )
)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let opts = match options::Options::from_derive_input(&ast) {
        Ok(val) => val,
        Err(err) => {
            return err.write_errors().into();
        }
    };
    builder::Builder::from(opts).to_token_stream().into()
}
