#![no_std]
//! This crate provides the user with a macro, `optional_struct`, which allows the user to generate
//! structures with optional fields, as well as functions for "fusing" such structures to form a
//! concrete instance of the original, user-defined structure. This was developed with the goal of
//! simplifying aggregating configurations coming from different sources, such as e.g. file, env,
//! CLI, etc.

/// The core of this crate. Call this proc macro on your structures to generate another structure
/// containing `Option`al fields, as well as helpers functions to convert those optional_struct to
/// their base, or even update only fields that have been set. This makes aggregating structures
/// from different sources (e.g. configuration from file/env/CLI) simple.
/// The generated struct by default will wrap all fields in an `Option`, unless the field already
/// is an `Option`. There are however other attributes that one can use to enforce a different
/// behaviour:
/// optional_rename => rename the type in the generated structure. Useful when the nested structure
/// itself has an optional_struct. This enables arbitrary nesting of optional_struct (see tests for
/// examples).
/// optional_skip_wrap => this force *not* wrapping a value, e.g. `T` stays `T`. This is enabled by
/// default if `T` is a `Optiona<U>`.
/// optional_wrap => this forces wrapping a value, e.g. `U` becomes `Option<U>`. Enabling this
/// allows nested `Option`, e.g. `Option<V>` can become `Option<Option<V>>`
/// optional_serde_skip_none => This generate an extra `#[serde(skip_serializing_if = ... )]` to the
/// generated structures. Useful if you want to (de)serialize those structures with serde.
pub use optional_struct_macro::optional_struct;

/// The trait is implemented for every generated structure. Thanks to this, you can use
/// optional_struct in generic contexts.
/// You should never have to implement this manually.
pub trait Applicable: Sized {
    /// This is the type the optional_struct macro was used on. We need the type to be able to
    /// generate methods generating such structures.
    type Base;

    /// This function applies all the fields set in this structure to an instance of its Base.
    /// Note that this works recursively, enabling the use of nested optional_struct structures.
    fn build(self, mut base: Self::Base) -> Self::Base {
        self.apply_to(&mut base);
        base
    }

    /// Similar to `Applicable::build`, but takes the Base by reference.
    fn apply_to(self, base: &mut Self::Base);

    /// Applies the fields of this structure to another optional_struct.
    /// The fields on the "left" (from self) are applied iff they are set. E.g.:
    /// self.a == Some(Foo) and other.a == Some(Bar) => other.a == Some(Foo)
    /// self.a == None and other.a == Some(Bar) => other.a == Some(Bar)
    /// (where the arrow means, after calling this function)
    /// This function is also called recursively, and supports nested structures.
    fn apply_to_opt(self, other: &mut Self);

    /// Similar to `apply_to_opt` but the argument `other` is applied to self. This allows chaining
    /// calls.
    fn apply(mut self, other: Self) -> Self {
        other.apply_to_opt(&mut self);
        self
    }

    /// Signals whether the optional_struct has all its fields set to convert it to a Base.
    /// i.e. self.can_convert() == Base::try_from(self).is_ok()
    fn can_convert(&self) -> bool;
}
