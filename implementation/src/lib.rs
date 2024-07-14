use std::collections::HashSet;

use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_quote, spanned::Spanned, Attribute, Data, DeriveInput, Field, Fields, Ident, Path, Token,
    Type, Visibility,
};

const RENAME_ATTRIBUTE: &str = "optional_rename";
const SKIP_WRAP_ATTRIBUTE: &str = "optional_skip_wrap";
const WRAP_ATTRIBUTE: &str = "optional_wrap";
const SERDE_SKIP_SERIALIZING_NONE: &str = "optional_serde_skip_none";
const CFG_ATTRIBUTE: &str = "cfg";

#[cfg(test)]
mod test;

struct FieldOptions {
    wrapping_behavior: bool,
    serde_skip: bool,
    cfg_attribute: Option<Attribute>,
    new_type: Option<TokenTree>,
    field_ident: TokenStream,
}

trait OptionalFieldVisitor {
    fn visit(
        &mut self,
        global_options: &GlobalOptions,
        old_field: &mut Field,
        new_field: &mut Field,
        field_options: &FieldOptions,
    );
}

struct GenerateCanConvertImpl {
    acc: TokenStream,
}

impl GenerateCanConvertImpl {
    fn new() -> Self {
        GenerateCanConvertImpl { acc: quote! {} }
    }

    fn get_implementation(self, derive_input: &DeriveInput, new: &DeriveInput) -> TokenStream {
        let (impl_generics, ty_generics, _) = derive_input.generics.split_for_impl();
        let new_name = &new.ident;
        let acc = self.acc;

        quote! {
            impl #impl_generics #new_name #ty_generics {
                fn can_convert(&self) -> bool {
                    #acc
                    true
                }
            }
        }
    }
}

impl OptionalFieldVisitor for GenerateCanConvertImpl {
    fn visit(
        &mut self,
        _global_options: &GlobalOptions,
        old_field: &mut Field,
        _new_field: &mut Field,
        field_options: &FieldOptions,
    ) {
        let ident = &field_options.field_ident;
        let cfg_attr = &field_options.cfg_attribute;

        let is_wrapped = field_options.wrapping_behavior;
        let is_nested = field_options.new_type.is_some();
        let is_base_opt = is_type_option(&old_field.ty);
        let inc = match (is_base_opt, is_wrapped, is_nested) {
            (_, true, false) => quote! { self.#ident.is_some() },
            (_, true, true) => {
                quote! { if let Some(i) = &self.#ident { !i.can_convert() } else { false } }
            }
            (_, false, true) => quote! { self.#ident.can_convert() },
            (_, false, false) => quote! { true },
        };
        let acc = &self.acc;
        self.acc = quote! {
            #acc
            #cfg_attr
            if !#inc {
                return false;
            }
        };
    }
}

struct GenerateTryFromImpl {
    field_assign_acc: TokenStream,
    field_check_acc: TokenStream,
}

impl GenerateTryFromImpl {
    fn new() -> Self {
        GenerateTryFromImpl {
            field_check_acc: quote! {},
            field_assign_acc: quote! {},
        }
    }

    fn get_implementation(self, derive_input: &DeriveInput, new: &DeriveInput) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();
        let old_name = &derive_input.ident;
        let new_name = &new.ident;
        let field_check_acc = self.field_check_acc;
        let field_assign_acc = self.field_assign_acc;

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
}

impl OptionalFieldVisitor for GenerateTryFromImpl {
    fn visit(
        &mut self,
        _global_options: &GlobalOptions,
        old_field: &mut Field,
        _new_field: &mut Field,
        field_options: &FieldOptions,
    ) {
        let ident = &field_options.field_ident;
        let cfg_attr = &field_options.cfg_attribute;

        let is_wrapped = field_options.wrapping_behavior;
        let is_nested = field_options.new_type.is_some();
        let is_base_opt = is_type_option(&old_field.ty);
        let (unwrap, check) = match (is_base_opt, is_wrapped, is_nested) {
            (_, true, false) => (
                quote! { .unwrap() },
                quote! { #cfg_attr if v.#ident.is_none() { return Err(v); } },
            ),
            (_, true, true) => (
                quote! { .unwrap().try_into().unwrap() },
                quote! { #cfg_attr if let Some(i) = &v.#ident { if !i.can_convert() { return Err(v); } } else { return Err(v); } },
            ),
            (_, false, true) => (
                quote! { .try_into().unwrap() },
                quote! { #cfg_attr if !v.#ident.can_convert() { return Err(v); } },
            ),
            (_, false, false) => (quote! {}, quote! {}),
        };

        let field_assign_acc = &self.field_assign_acc;
        self.field_assign_acc = quote! {
            #field_assign_acc
            #cfg_attr

            #ident: v.#ident #unwrap,
        };

        let field_check_acc = &self.field_check_acc;
        self.field_check_acc = quote! {
            #field_check_acc
            #check
        };
    }
}

struct GenerateApplyFnVisitor {
    acc_concrete: TokenStream,
    acc_opt: TokenStream,
}

impl GenerateApplyFnVisitor {
    fn new() -> Self {
        GenerateApplyFnVisitor {
            acc_concrete: quote! {},
            acc_opt: quote! {},
        }
    }

    fn get_implementation(self, orig: &DeriveInput, new: &DeriveInput) -> TokenStream {
        let (impl_generics, ty_generics, _) = orig.generics.split_for_impl();
        let orig_name = &orig.ident;
        let new_name = &new.ident;
        let acc_concrete = self.acc_concrete;
        let acc_opt = self.acc_opt;
        // TODO: everything was written with "t" as the parameter name, but this a. does not match
        // the trait and b. is not explicit enough. Make this some parameter instead.
        quote! {
            impl #impl_generics optional_struct::Applicable for #new_name #ty_generics {
                type Base = #orig_name #ty_generics;

                fn apply_to(self, t: &mut Self::Base) {
                    #acc_concrete
                }

                fn apply_to_opt(self, t: &mut Self) {
                    #acc_opt
                }
            }
        }
    }

    fn get_incremental_setter_concrete(
        ident: &TokenStream,
        is_wrapped: bool,
        is_nested: bool,
        is_base_opt: bool,
    ) -> TokenStream {
        match (is_base_opt, is_wrapped, is_nested) {
            (true, false, true) => quote! {
               match (&mut t.#ident, self.#ident) {
                   (None, Some(nested)) => t.#ident = nested.#ident.try_into(),
                   (Some(existing), Some(nested)) => nested.#ident.apply_to(existing),
                   (_, None) => {},
               }
            },
            (true, false, false) => quote! {
                if self.#ident.is_some() {
                    t.#ident = self.#ident;
                }
            },
            (false, false, true) => quote! { self.#ident.apply_to(&mut t.#ident); },
            (false, false, false) => quote! { t.#ident = self.#ident; },
            (_, true, true) => {
                quote! { if let Some(inner) = self.#ident { inner.apply_to(&mut t.#ident); } }
            }
            (_, true, false) => quote! { if let Some(inner) = self.#ident { t.#ident = inner; } },
        }
    }
    fn get_incremental_setter_opt(
        ident: &TokenStream,
        is_wrapped: bool,
        is_nested: bool,
        is_base_opt: bool,
    ) -> TokenStream {
        match (is_base_opt, is_wrapped, is_nested) {
            (true, false, true) => quote! {
               match (&mut t.#ident, self.#ident) {
                   (None, Some(nested)) => t.#ident = Some(nested),
                   (Some(existing), Some(nested)) => nested.apply_to_opt(existing),
                   (_, None) => {},
               }
            },
            (true, false, false) => quote! {
                if self.#ident.is_some() {
                    t.#ident = self.#ident;
                }
            },
            (false, false, true) => quote! { self.#ident.apply_to_opt(&mut t.#ident); },
            (false, false, false) => quote! { t.#ident = self.#ident; },
            (_, true, true) => {
                quote! { if let Some(inner) = self.#ident { inner.apply_to_opt(&mut t.#ident); } }
            }
            (_, true, false) => quote! { if let Some(inner) = self.#ident { t.#ident = inner; } },
        }
    }
}

impl OptionalFieldVisitor for GenerateApplyFnVisitor {
    fn visit(
        &mut self,
        _global_options: &GlobalOptions,
        old_field: &mut Field,
        _new_field: &mut Field,
        field_options: &FieldOptions,
    ) {
        let ident = &field_options.field_ident;
        let cfg_attr = &field_options.cfg_attribute;

        let is_wrapped = field_options.wrapping_behavior;
        let is_nested = field_options.new_type.is_some();
        let is_base_opt = is_type_option(&old_field.ty);

        let inc_concrete =
            Self::get_incremental_setter_concrete(ident, is_wrapped, is_nested, is_base_opt);
        // Opt <-> Opt is never wrapped. But both have an Option<> if the initial type IS wrapped!
        let inc_opt =
            Self::get_incremental_setter_opt(ident, false, is_nested, is_wrapped || is_base_opt);

        let acc_concrete = &self.acc_concrete;
        self.acc_concrete = quote! {
            #acc_concrete

            #cfg_attr
            #inc_concrete
        };

        let acc_opt = &self.acc_opt;
        self.acc_opt = quote! {
            #acc_opt

            #cfg_attr
            #inc_opt
        };
    }
}

struct SetNewFieldVisibilityVisitor;

impl OptionalFieldVisitor for SetNewFieldVisibilityVisitor {
    fn visit(
        &mut self,
        global_options: &GlobalOptions,
        _old_field: &mut Field,
        new_field: &mut Field,
        _field_options: &FieldOptions,
    ) {
        if global_options.make_fields_public {
            new_field.vis = Visibility::Public(syn::token::Pub(new_field.vis.span()))
        }
    }
}

struct SetNewFieldTypeVisitor;

impl OptionalFieldVisitor for SetNewFieldTypeVisitor {
    fn visit(
        &mut self,
        _global_options: &GlobalOptions,
        old_field: &mut Field,
        new_field: &mut Field,
        field_options: &FieldOptions,
    ) {
        let mut new_type = if let Some(t) = &field_options.new_type {
            quote! {#t}
        } else {
            let t = &old_field.ty;
            quote! {#t}
        };

        if field_options.wrapping_behavior {
            new_type = quote! {Option<#new_type>};
        };
        new_field.ty = Type::Verbatim(new_type);
    }
}

struct AddSerdeSkipAttribute;

impl OptionalFieldVisitor for AddSerdeSkipAttribute {
    fn visit(
        &mut self,
        _global_options: &GlobalOptions,
        _old_field: &mut Field,
        new_field: &mut Field,
        field_options: &FieldOptions,
    ) {
        if !field_options.serde_skip {
            return;
        }

        let attribute: Attribute =
            parse_quote! { #[serde(skip_serializing_if = "Option::is_none")] };
        new_field.attrs.push(attribute);
    }
}

// https://github.com/rust-lang/rust/issues/65823 :(
struct RemoveHelperAttributesVisitor;

impl OptionalFieldVisitor for RemoveHelperAttributesVisitor {
    fn visit(
        &mut self,
        _global_options: &GlobalOptions,
        old_field: &mut Field,
        new_field: &mut Field,
        _field_options: &FieldOptions,
    ) {
        let indexes_to_remove = old_field
            .attrs
            .iter()
            .enumerate()
            .filter_map(|(i, a)| {
                if a.path().is_ident(RENAME_ATTRIBUTE) {
                    Some(i)
                } else if a.path().is_ident(SKIP_WRAP_ATTRIBUTE) {
                    Some(i)
                } else if a.path().is_ident(WRAP_ATTRIBUTE) {
                    Some(i)
                } else if a.path().is_ident(SERDE_SKIP_SERIALIZING_NONE) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Don't forget to reverse so the indices are removed without being shifted!
        for i in indexes_to_remove.into_iter().rev() {
            old_field.attrs.swap_remove(i);
            new_field.attrs.swap_remove(i);
        }
    }
}

fn borrow_fields(derive_input: &mut DeriveInput) -> &mut Punctuated<Field, Comma> {
    let data_struct = match &mut derive_input.data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("OptionalStruct only works for structs :)"),
    };

    match &mut data_struct.fields {
        Fields::Unnamed(f) => &mut f.unnamed,
        Fields::Named(f) => &mut f.named,
        Fields::Unit => unreachable!("A struct cannot have simply a unit field?"),
    }
}

fn visit_fields(
    visitors: &mut [&mut dyn OptionalFieldVisitor],
    global_options: &GlobalOptions,
    derive_input: &DeriveInput,
) -> (DeriveInput, DeriveInput) {
    let mut new = derive_input.clone();
    let mut orig = derive_input.clone();
    let old_fields = borrow_fields(&mut orig);
    let new_fields = borrow_fields(&mut new);

    for (struct_index, (old_field, new_field)) in
        old_fields.iter_mut().zip(new_fields.iter_mut()).enumerate()
    {
        let mut wrapping_behavior =
            !is_type_option(&old_field.ty) && global_options.default_wrapping_behavior;
        let mut cfg_attribute = None;
        let mut new_type = None;
        let mut serde_skip = false;
        old_field.attrs
            .iter()
            .for_each(|a| {
                if a.path().is_ident(RENAME_ATTRIBUTE) {
                    let args = a
                        .parse_args()
                        .expect(&format!("'{RENAME_ATTRIBUTE}' attribute expects one and only one argument (the new type to use)"));
                    new_type = Some(args);
                    wrapping_behavior = false;
                } else if a.path().is_ident(SKIP_WRAP_ATTRIBUTE) {
                    wrapping_behavior = false;
                } else if a.path().is_ident(WRAP_ATTRIBUTE) {
                    wrapping_behavior = true;
                } else if a.path().is_ident(SERDE_SKIP_SERIALIZING_NONE) {
                    serde_skip = true;
                } else if a.path().is_ident(CFG_ATTRIBUTE) {
                    cfg_attribute = Some(a.clone());
                }
            });
        let field_ident = if let Some(ident) = &old_field.ident {
            quote! {#ident}
        } else {
            let i = syn::Index::from(struct_index);
            quote! {#i}
        };
        let field_options = FieldOptions {
            wrapping_behavior,
            cfg_attribute,
            new_type,
            field_ident,
            serde_skip,
        };
        for v in &mut *visitors {
            v.visit(&global_options, old_field, new_field, &field_options);
        }
    }
    (orig, new)
}

fn get_derive_macros(new: &DeriveInput, extra_derive: &[String]) -> TokenStream {
    let mut extra_derive = extra_derive.iter().collect::<HashSet<_>>();
    for attributes in &new.attrs {
        let _ = attributes.parse_nested_meta(|derived_trait| {
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

struct GlobalOptions {
    new_struct_name: String,
    extra_derive: Vec<String>,
    default_wrapping_behavior: bool,
    make_fields_public: bool,
}

impl GlobalOptions {
    fn new(attr: ParsedMacroParameters, struct_definition: &DeriveInput) -> Self {
        let new_struct_name = attr
            .new_struct_name
            .unwrap_or_else(|| "Optional".to_owned() + &struct_definition.ident.to_string());
        let default_wrapping_behavior = attr.default_wrapping;
        GlobalOptions {
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

pub struct OptionalStructOutput {
    pub original: TokenStream,
    pub generated: TokenStream,
}

pub fn opt_struct(attr: TokenStream, input: TokenStream) -> OptionalStructOutput {
    let derive_input = syn::parse2::<DeriveInput>(input).unwrap();
    let macro_params = GlobalOptions::new(syn::parse2::<_>(attr).unwrap(), &derive_input);

    let mut apply_fn_generator = GenerateApplyFnVisitor::new();
    let mut try_from_generator = GenerateTryFromImpl::new();
    let mut can_convert_generator = GenerateCanConvertImpl::new();

    let mut visitors = [
        &mut RemoveHelperAttributesVisitor as &mut dyn OptionalFieldVisitor,
        &mut SetNewFieldVisibilityVisitor,
        &mut SetNewFieldTypeVisitor,
        &mut AddSerdeSkipAttribute,
        &mut apply_fn_generator,
        &mut try_from_generator,
        &mut can_convert_generator,
    ];

    let (orig, mut new) = visit_fields(&mut visitors, &macro_params, &derive_input);

    new.ident = Ident::new(&macro_params.new_struct_name, new.ident.span());

    let apply_fn_impl = apply_fn_generator.get_implementation(&derive_input, &new);
    let try_from_impl = try_from_generator.get_implementation(&derive_input, &new);
    let can_convert_impl = can_convert_generator.get_implementation(&derive_input, &new);

    let derives = get_derive_macros(&new, &macro_params.extra_derive);

    let generated = quote! {
        #derives
        #new
        #apply_fn_impl
        #try_from_impl
        #can_convert_impl
    };

    OptionalStructOutput {
        original: quote! { #orig },
        generated,
    }
}
