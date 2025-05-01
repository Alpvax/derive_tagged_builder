extern crate derive_tagged_builder_macro;

#[doc(inline)]
pub use derive_tagged_builder_macro::TaggedBuilder;

pub struct UnspecifiedProperty;
pub struct SpecifiedProperty;
pub trait PropertySpecifiedTag {}
impl PropertySpecifiedTag for UnspecifiedProperty {}
impl PropertySpecifiedTag for SpecifiedProperty {}
