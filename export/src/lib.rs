use optional_struct_macro_impl::opt_struct;
use quote::quote;

#[proc_macro_attribute]
pub fn optional_struct(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let out = opt_struct(attr.into(), input.into());
    let original = out.original;
    let generated = out.generated;
    proc_macro::TokenStream::from(quote! {
        #original

        #generated
    })
}

