use std::collections::HashSet;

use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Data, DeriveInput, Field, Fields, Ident, Path, spanned::Spanned, Token, Type, Visibility};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;

const RENAME_ATTRIBUTE: &str = "optional_rename";
const SKIP_WRAP_ATTRIBUTE: &str = "optional_skip_wrap";
const WRAP_ATTRIBUTE: &str = "optional_wrap";

#[cfg(test)]
mod test;

struct DeriveInputWrapper {
    orig: DeriveInput,
    new: DeriveInput,
}

enum WhichStruct {
    Original,
    New,
}

impl DeriveInputWrapper {
    fn new(derive_input: DeriveInput) -> Self {
        DeriveInputWrapper { orig: derive_input.clone(), new: derive_input }
    }

    fn original(&self) -> &DeriveInput {
        &self.orig
    }

    fn new_struct(&self) -> &DeriveInput {
        &self.new
    }

    fn set_new_struct_fields(&mut self, macro_params: &MacroParameters) {
        self.iter_struct_fields(WhichStruct::New, macro_params.into())
    }

    fn set_new_name(&mut self, new_name: &str) {
        self.new.ident = Ident::new(new_name, self.new.ident.span());
    }

    fn get_derive_macros(
        &self,
        extra_derive: &[String],
    ) -> TokenStream {
        let mut extra_derive = extra_derive.iter().collect::<HashSet<_>>();
        for attributes in &self.new.attrs {
            let _ = attributes.parse_nested_meta(|derived_trait|
                {
                    let derived_trait = derived_trait.path;
                    let full_path = quote! { #derived_trait };
                    extra_derive.remove(&full_path.to_string());
                    Ok(())
                });
        }


        let mut acc = quote! {};
        for left_trait_to_derive in extra_derive {
            let left_trait_to_derive = format_ident!("{left_trait_to_derive}");
            acc = quote! { # left_trait_to_derive, # acc};
        }

        quote! { #[derive(#acc)] }
    }

    fn fields(&self, which_struct: WhichStruct) -> &Punctuated<Field, Comma> {
        let s = match which_struct {
            WhichStruct::Original => &self.orig,
            WhichStruct::New => &self.new,
        };

        let data_struct = match &s.data {
            Data::Struct(data_struct) => data_struct,
            _ => panic!("OptionalStruct only works for structs :)"),
        };

        match &data_struct.fields {
            Fields::Unnamed(f) => &f.unnamed,
            Fields::Named(f) => &f.named,
            Fields::Unit => unreachable!("A struct cannot have simply a unit field?"),
        }
    }

    fn fields_mut(&mut self, which_struct: WhichStruct) -> &mut Punctuated<Field, Comma> {
        let s = match which_struct {
            WhichStruct::Original => &mut self.orig,
            WhichStruct::New => &mut self.new,
        };

        let data_struct = match &mut s.data {
            Data::Struct(data_struct) => data_struct,
            _ => panic!("OptionalStruct only works for structs :)"),
        };

        match &mut data_struct.fields {
            Fields::Unnamed(f) => &mut f.unnamed,
            Fields::Named(f) => &mut f.named,
            Fields::Unit => unreachable!("A struct cannot have simply a unit field?"),
        }
    }


    fn iter_struct_fields(&mut self, which_struct: WhichStruct, new_struct_params: Option<&MacroParameters>) {
        // TODO: has to be a cleaner way
        let (apply_attribute_metadata, default_wrapping, make_fields_public) = if let Some(n) = new_struct_params {
            (true, n.default_wrapping_behavior, n.make_fields_public)
        } else {
            (false, false, false)
        };

        for field in self.fields_mut(which_struct).iter_mut() {
            let field_meta_data = extract_relevant_attributes(field, default_wrapping);
            if apply_attribute_metadata {
                field_meta_data.apply_to_field(field);
                if make_fields_public {
                    field.vis = Visibility::Public(syn::token::Pub(field.vis.span()))
                }
            }
        }
    }

    fn finalize_definition(mut self, macro_parameters: &MacroParameters) -> (TokenStream, TokenStream) {
        // https://github.com/rust-lang/rust/issues/65823 :(
        self.iter_struct_fields(WhichStruct::Original, None);

        let derives = self.get_derive_macros(&macro_parameters.extra_derive);

        let orig = self.orig;
        let new = self.new;
        (quote! { #orig }, quote! { #derives #new })
    }
}

struct ParsedMacroParameters {
    new_struct_name: Option<String>,
    default_wrapping: bool,
}

impl Parse for ParsedMacroParameters {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut out = ParsedMacroParameters {
            new_struct_name: None,
            default_wrapping: true,
        };

        if let Ok(struct_name) = Ident::parse(input) {
            out.new_struct_name = Some(struct_name.to_string());
        } else {
            return Ok(out);
        };

        if input.parse::<Token![,]>().is_err() {
            return Ok(out);
        };

        if let Ok(wrapping) = syn::LitBool::parse(input) {
            out.default_wrapping = wrapping.value;
        } else {
            return Ok(out);
        };

        Ok(out)
    }
}

// TODO this breaks for e.g. yolo::my::Option
fn is_path_option(p: &Path) -> bool {
    p.segments
        .last()
        .map(|ps| ps.ident == "Option")
        .unwrap_or(false)
}

fn is_type_option(t: &Type) -> bool {
    macro_rules! wtf {
        ($reason : tt) => {
            panic!(
                "Using OptionalStruct for a struct containing a {} is dubious...",
                $reason
            )
        };
    }

    match &t {
        // real work
        Type::Path(type_path) => is_path_option(&type_path.path),
        Type::Array(_) | Type::Tuple(_) => false,
        Type::Paren(type_paren) => is_type_option(&type_paren.elem),

        // No clue what to do with those
        Type::ImplTrait(_) | Type::TraitObject(_) => {
            panic!("Might already be an option I have no way to tell :/")
        }
        Type::Infer(_) => panic!("If you cannot tell, neither can I"),
        Type::Macro(_) => panic!("Don't think I can handle this easily..."),

        // Makes no sense to use those in an OptionalStruct
        Type::Reference(_) => wtf!("reference"),
        Type::Never(_) => wtf!("never-type"),
        Type::Slice(_) => wtf!("slice"),
        Type::Ptr(_) => wtf!("pointer"),
        Type::BareFn(_) => wtf!("function pointer"),

        // Help
        Type::Verbatim(_) => todo!("Didn't get what this was supposed to be..."),
        Type::Group(_) => todo!("Not sure what to do here"),

        // Have to wildcard here but I don't want to (unneeded as long as syn doesn't break semver
        // anyway)
        _ => panic!("Open an issue please :)"),
    }
}

struct MacroParameters {
    new_struct_name: String,
    extra_derive: Vec<String>,
    default_wrapping_behavior: bool,
    make_fields_public: bool,
}

impl MacroParameters {
    fn new(attr: ParsedMacroParameters, struct_definition: &DeriveInput) -> Self {
        let new_struct_name = attr.new_struct_name.unwrap_or_else(|| "Optional".to_owned() + &struct_definition.ident.to_string());
        let default_wrapping_behavior = attr.default_wrapping;
        MacroParameters {
            new_struct_name,
            extra_derive: vec!["Clone", "PartialEq", "Default", "Debug"]
                .into_iter()
                .map(|s| s.to_owned())
                .collect(),
            default_wrapping_behavior,
            make_fields_public: true,
        }
    }
}


struct FieldAttributeData {
    wrap: bool,
    new_type: Option<TokenTree>,
}

impl FieldAttributeData {
    fn apply_to_field(self, f: &mut Field) {
        let mut new_type = if let Some(t) = self.new_type {
            quote! {#t}
        } else {
            let t = &f.ty;
            quote! {#t}
        };

        if self.wrap {
            new_type = quote! {Option<#new_type>};
        };
        f.ty = Type::Verbatim(new_type);
    }
}

fn extract_relevant_attributes(field: &mut Field, default_wrapping: bool) -> FieldAttributeData {
    let mut field_attribute_data = FieldAttributeData {
        wrap: !is_type_option(&field.ty) && default_wrapping,
        new_type: None,
    };
    let indexes_to_remove = field
        .attrs
        .iter()
        .enumerate()
        .filter_map(|(i, a)| {
            if a.path().is_ident(RENAME_ATTRIBUTE) {
                let args = a
                    .parse_args()
                    .expect("'{RENAME_ATTRIBUTE}' attribute expects one and only one argument (the new type to use)");
                field_attribute_data.new_type = Some(args);
                Some(i)
            } else if a.path().is_ident(SKIP_WRAP_ATTRIBUTE) {
                field_attribute_data.wrap = false;
                Some(i)
            } else if a.path().is_ident(WRAP_ATTRIBUTE) {
                field_attribute_data.wrap = true;
                Some(i)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // Don't forget to reverse so the indices are removed without being shifted!
    for i in indexes_to_remove.into_iter().rev() {
        field.attrs.swap_remove(i);
    }
    field_attribute_data
}

fn acc_assigning<'a, T: Iterator<Item=(U, &'a Vec<Attribute>)>, U: std::borrow::Borrow<V>, V: ToTokens>(
    idents_with_attrs: T,
) -> TokenStream {
    let mut acc = quote! {};
    for (ident, attrs) in idents_with_attrs {
        let ident = ident.borrow();
        let cfg_attr = attrs.iter().find(|attr| attr.path().is_ident("cfg"));
        acc = quote! {
            #acc

            #cfg_attr
            self.#ident.apply_to(&mut t.#ident);
        };
    }
    acc
}

fn acc_check<'a, T: Iterator<Item=(U, &'a Vec<Attribute>)>, U: std::borrow::Borrow<V>, V: ToTokens>(
    idents_with_attrs: T,
) -> TokenStream {
    let mut acc = quote! {};
    for (ident, attrs) in idents_with_attrs {
        let ident = ident.borrow();
        let cfg_attr = attrs.iter().find(|attr| attr.path().is_ident("cfg"));
        acc = quote! {
            #acc

            #cfg_attr
            if !self.#ident.can_be_applied() { return false; }
        };
    }
    acc
}

fn generate_apply_fn(
    derive_input: &DeriveInputWrapper,
) -> TokenStream {
    let orig_name = &derive_input.original().ident;
    let new_name = &derive_input.new_struct().ident;

    let fields = match &derive_input.original().data {
        Data::Struct(s) => &s.fields,
        _ => unreachable!(),
    };

    let apply_to_acc = match &fields {
        Fields::Unit => unreachable!(),
        Fields::Named(fields_named) => {
            let it = fields_named.named.iter().map(|f| (f.ident.as_ref().unwrap(), &f.attrs));
            acc_assigning::<_, _, Ident>(it)
        }
        Fields::Unnamed(fields_unnamed) => {
            let it = fields_unnamed.unnamed.iter().enumerate().map(|(i, field)| {
                let i = syn::Index::from(i);
                (quote! {#i}, &field.attrs)
            });
            acc_assigning(it)
        }
    };

    // TODO: copy pasted code
    let can_apply_acc = match &fields {
        Fields::Unit => unreachable!(),
        Fields::Named(fields_named) => {
            let it = fields_named.named.iter().map(|f| (f.ident.as_ref().unwrap(), &f.attrs));
            acc_check::<_, _, Ident>(it)
        }
        Fields::Unnamed(fields_unnamed) => {
            let it = fields_unnamed.unnamed.iter().enumerate().map(|(i, field)| {
                let i = syn::Index::from(i);
                (quote! {#i}, &field.attrs)
            });
            acc_check(it)
        }
    };

    let (impl_generics, ty_generics, where_clause) = derive_input.original().generics.split_for_impl();
    quote! {
        /*
        impl #impl_generics Applyable<#orig_name #ty_generics> #where_clause for Option<#new_name #ty_generics >{
            fn apply_to(self, t: &mut #orig_name #ty_generics) {
                if let Some(s) = self {
                    s.apply_to(t);
                }
            }
        }
        */

        impl #impl_generics Applyable<#orig_name #ty_generics> #where_clause for #new_name #ty_generics {
            fn apply_to(self, t: &mut #orig_name #ty_generics) {
                #apply_to_acc
            }

            fn can_be_applied(&self) -> bool {
                #can_apply_acc
                true
            }
        }
    }
}

fn generate_try_from_impl(derive_input: &DeriveInputWrapper, macro_parameters: &MacroParameters) -> TokenStream {
    let old_name = &derive_input.original().ident;
    let new_name = &derive_input.new_struct().ident;

    let (impl_generics, ty_generics, where_clause) = derive_input.new_struct().generics.split_for_impl();

    let mut field_check_acc = quote! { };
    let mut field_assign_acc = quote! { };
    for (i, field) in derive_input.fields(WhichStruct::Original).iter().enumerate() {
        let is_wrapped = field.attrs
            .iter()
            .rev().find_map(
            |a| {
                if a.path().is_ident(SKIP_WRAP_ATTRIBUTE) {
                    Some(false)
                } else if a.path().is_ident(WRAP_ATTRIBUTE) {
                    Some(true)
                } else {
                    None
                }
            }).unwrap_or(!is_type_option(&field.ty) && macro_parameters.default_wrapping_behavior);

        let ident = field.ident.as_ref().map(|id| quote! { #id }).unwrap_or_else(|| {
            let index = syn::Index::from(i);
            quote! { #index }
        });

        // TODO: we're looping twice on this
        let cfg_attr = field.attrs.iter().find(|attr| attr.path().is_ident("cfg"));

        let (unwrap, check) = if is_wrapped {
            (
                quote!{ .unwrap() },
                quote!{ #cfg_attr if v.#ident.is_none() { return Err(v); } }
            )
        } else {
            (
            quote!{},
            quote!{}
            )
        };


        field_assign_acc = quote! {
            #field_assign_acc

            #cfg_attr
            #ident: v.#ident #unwrap,
        };

        field_check_acc = quote!{
            #field_check_acc

            #check
        };
    }

    quote! {
        impl #impl_generics TryFrom<#new_name #ty_generics > #where_clause for #old_name #ty_generics {
            type Error = #new_name #ty_generics;

            fn try_from(v: Self::Error) -> Result<Self, Self::Error> {
                #field_check_acc
                Ok(Self {
                    #field_assign_acc
                })
            }
        }
    }
}


pub struct OptionalStructOutput {
    pub original: TokenStream,
    pub generated: TokenStream,
}

pub fn opt_struct(
    attr: TokenStream,
    input: TokenStream,
) -> OptionalStructOutput {
    let mut derive_input = DeriveInputWrapper::new(syn::parse2::<DeriveInput>(input).unwrap());
    let macro_params = MacroParameters::new(syn::parse2::<_>(attr).unwrap(), derive_input.original());

    derive_input.set_new_name(&macro_params.new_struct_name);
    derive_input.set_new_struct_fields(&macro_params);
    let apply_fn_impl = generate_apply_fn(&derive_input);
    let try_from_impl = generate_try_from_impl(&derive_input, &macro_params);

    let (original, new) = derive_input.finalize_definition(&macro_params);

    OptionalStructOutput {
        original,
        generated: quote! {
            #new

            #apply_fn_impl

            #try_from_impl
        },
    }
}