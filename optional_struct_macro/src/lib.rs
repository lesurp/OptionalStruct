use quote::quote;

#[cfg(test)]
mod test;
mod opt_struct;


#[proc_macro_attribute]
pub fn optional_struct(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let out = opt_struct::opt_struct(attr.into(), input.into());
    let original = out.original;
    let generated = out.generated;
    proc_macro::TokenStream::from(quote! {
        #original

        #generated
    })
}
