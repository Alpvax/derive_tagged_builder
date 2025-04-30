use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Parser, Attribute, GenericParam, Generics, Ident, Type, Visibility};

use crate::options::{BuildFnDef, FieldRequirement, FnGenerationSpec, GenericArg};

#[derive(Debug, Clone)]
pub(crate) struct FieldSetterDef {
    /// The property name
    prop: Ident,
    /// The property type
    typ: Type,
    /// Whether or not the field is required in order to build the struct
    required: FieldRequirement,
    /// Whether the setter can be called more than once
    override_allowed: bool,
    /// The name of the (self, Into<value>) -> Self setter, or none to not generate it
    setter_ident_owned: Option<Ident>,
    /// The name of the (&mut self, Into<value>) -> &mut Self setter, or none to not generate it
    setter_ident_borrowed: Option<Ident>,
    /// The name of the (&mut self, TryInto<value>) -> Result<(), TryInto::Err> setter, or none to not generate it
    setter_ident_try: Option<Ident>,
}

#[derive(Debug, Clone)]
struct GenericsWrapper<'g> {
    built: &'g Generics,
    fields: Vec<GenericArg>,
}
fn specified_param_tokens(generic: &GenericArg) -> TokenStream {
    match generic {
        GenericArg::Name(_) => quote! { ::derive_tagged_builder::SpecifiedProperty },
        GenericArg::Const(_, Type::Path(syn::TypePath { path, .. })) => {
            match darling::util::path_to_string(path).as_str() {
                "bool" => quote! { true },
                "usize" | "u8" | "u16" | "u32" | "u64" | "u128" => quote! { 1 },
                p => abort_call_site!("Unsupported const path type: {}", p),
            }
        }
        GenericArg::Const(_, typ) => abort_call_site!("Unexpected const type: {:?}", typ),
    }
}
fn unspecified_param_tokens(generic: &GenericArg) -> TokenStream {
    match generic {
        GenericArg::Name(_) => quote! { ::derive_tagged_builder::UnspecifiedProperty },
        GenericArg::Const(_, Type::Path(syn::TypePath { path, .. })) => {
            match darling::util::path_to_string(path).as_str() {
                "bool" => quote! { false },
                "usize" | "u8" | "u16" | "u32" | "u64" | "u128" => quote! { 0 },
                p => abort_call_site!("Unsupported const path type: {}", p),
            }
        }
        GenericArg::Const(_, typ) => abort_call_site!("Unexpected const type: {:?}", typ),
    }
}
fn generic_tokens(generic: &GenericArg) -> (TokenStream, TokenStream) {
    match generic {
        GenericArg::Name(name) => (name.to_token_stream(), name.to_token_stream()),
        GenericArg::Const(name, typ) => (quote! { const #name: #typ }, name.to_token_stream()),
    }
}
impl GenericsWrapper<'_> {
    fn new(generics: &Generics) -> GenericsWrapper {
        GenericsWrapper {
            built: generics,
            fields: Vec::new(),
        }
    }
    fn push(&mut self, generic: GenericArg) {
        self.fields.push(generic);
    }
    fn for_fields<F>(&self, check_req: F) -> (TokenStream, TokenStream, TokenStream)
    where
        F: Fn(usize) -> Option<bool>,
    {
        let lt_tok = self.built.lt_token.unwrap_or_default();
        let gt_tok = self.built.gt_token.unwrap_or_default();
        let lifetimes = self.built.lifetimes().collect::<Vec<_>>();
        let params = self
            .built
            .params
            .iter()
            .filter(|param| !matches!(param, GenericParam::Lifetime(..)))
            .collect::<Vec<_>>();
        let (impl_g, typ_g) = self.fields.iter().enumerate().fold(
            (Vec::new(), Vec::new()),
            |(mut impl_g, mut typ_g), (i, f)| {
                match check_req(i) {
                    Some(true) => {
                        typ_g.push(specified_param_tokens(f));
                    }
                    Some(false) => {
                        typ_g.push(unspecified_param_tokens(f));
                    }
                    None => {
                        let (i_g, t_g) = generic_tokens(f);
                        impl_g.push(i_g);
                        typ_g.push(t_g);
                    }
                }
                (impl_g, typ_g)
            },
        );
        (
            quote! { #lt_tok #(#lifetimes,)* #(#params,)* #(#impl_g),* #gt_tok },
            quote! { #lt_tok #(#lifetimes,)* #(#params,)* #(#typ_g),* #gt_tok },
            self.built.where_clause.to_token_stream(),
        )
    }
    fn all_specified(&self) -> (TokenStream, TokenStream, TokenStream) {
        let lt_tok = self.built.lt_token.unwrap_or_default();
        let gt_tok = self.built.gt_token.unwrap_or_default();
        let lifetimes = self.built.lifetimes().collect::<Vec<_>>();
        let params = self
            .built
            .params
            .iter()
            .filter(|param| !matches!(param, GenericParam::Lifetime(..)))
            .collect::<Vec<_>>();
        let typ_g = self
            .fields
            .iter()
            .map(specified_param_tokens)
            .collect::<Vec<_>>();
        (
            quote! { #lt_tok #(#lifetimes,)* #(#params,)* #gt_tok },
            quote! { #lt_tok #(#lifetimes,)* #(#params,)* #(#typ_g),* #gt_tok },
            self.built.where_clause.to_token_stream(),
        )
    }
    fn all_unspecified(&self) -> (TokenStream, TokenStream, TokenStream) {
        let lt_tok = self.built.lt_token.unwrap_or_default();
        let gt_tok = self.built.gt_token.unwrap_or_default();
        let lifetimes = self.built.lifetimes().collect::<Vec<_>>();
        let params = self
            .built
            .params
            .iter()
            .filter(|param| !matches!(param, GenericParam::Lifetime(..)))
            .collect::<Vec<_>>();
        let typ_g = self
            .fields
            .iter()
            .map(unspecified_param_tokens)
            .collect::<Vec<_>>();
        (
            quote! { #lt_tok #(#lifetimes,)* #(#params,)* #gt_tok },
            quote! { #lt_tok #(#lifetimes,)* #(#params,)* #(#typ_g),* #gt_tok },
            self.built.where_clause.to_token_stream(),
        )
    }
    fn all_generic(&self) -> (TokenStream, TokenStream, TokenStream) {
        let lt_tok = self.built.lt_token.unwrap_or_default();
        let gt_tok = self.built.gt_token.unwrap_or_default();
        let lifetimes = self.built.lifetimes().collect::<Vec<_>>();
        let params = self
            .built
            .params
            .iter()
            .filter(|param| !matches!(param, GenericParam::Lifetime(..)))
            .collect::<Vec<_>>();
        let (impl_g, typ_g): (Vec<_>, Vec<_>) = self.fields.iter().map(generic_tokens).unzip();
        (
            quote! { #lt_tok #(#lifetimes,)* #(#params,)* #(#impl_g),* #gt_tok },
            quote! { #lt_tok #(#lifetimes,)* #(#params,)* #(#typ_g),* #gt_tok },
            self.built.where_clause.to_token_stream(),
        )
    }
}

#[derive(Debug)]
pub(crate) struct Builder {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    built: Ident,
    generics_inherit: Generics,
    all_props: Vec<Ident>,
    setters: Vec<FieldSetterDef>,
    build_fn: BuildFnDef,
}
impl ToTokens for Builder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            attrs,
            vis,
            name,
            built,
            generics_inherit,
            all_props,
            setters,
            build_fn,
        } = self;
        let (field_generics, field_defs, fields_init, build_assignments) = setters.iter().fold((GenericsWrapper::new(generics_inherit), Vec::new(), Vec::new(), Vec::new()), |(mut field_generics, mut field_defs, mut field_init, mut build_assignments), FieldSetterDef { prop, typ, required, .. }| {
            match required {
                FieldRequirement::Required(generic) => {
                    field_generics.push(generic.clone());
                    field_defs.push(quote! { #prop: Option<#typ>, });
                    field_init.push(quote! { #prop: None, });
                    build_assignments.push(quote! { #prop: self.#prop.expect(concat!("Should not be able to call build when ", stringify!(#prop), " has not been set!")), });
                },
                FieldRequirement::Defaulted => {
                    field_defs.push(quote! { #prop: #typ, });
                    field_init.push(quote! { #prop: Default::default(), });
                    build_assignments.push(quote! { #prop: self.#prop, });
                },
                FieldRequirement::FallbackTo(expr) => {
                    field_defs.push(quote! { #prop: Option<#typ>, });
                    field_init.push(quote! { #prop: None, });
                    build_assignments.push(quote! { #prop: self.#prop.unwrap_or_else(#expr), });
                },
                FieldRequirement::FallbackFn(expr) =>  {
                    field_defs.push(quote! { #prop: Option<#typ>, });
                    field_init.push(quote! { #prop: None, });
                    build_assignments.push(quote! { #prop: self.#prop.unwrap_or_else(#expr()), });
                },
            }
            (field_generics, field_defs, field_init, build_assignments)
        });

        let (built_impl, built_typ, built_where) = generics_inherit.split_for_impl();
        let (generic_impl, _, generic_where) = field_generics.all_generic();
        let (unspec_impl, unspec_typ, unspec_where) = field_generics.all_unspecified();
        let (spec_impl, spec_typ, spec_where) = field_generics.all_specified();
        // Struct def, unimplemented factory impls
        tokens.extend(quote! {
            #(#attrs)*
            #vis struct #name #generic_impl #generic_where {
                #(#field_defs)*
            }
            impl #unspec_impl Default for #name #unspec_typ #unspec_where {
                fn default() -> Self {
                    Self {
                        #(#fields_init)*
                    }
                }
            }
            impl #unspec_impl #name #unspec_typ #unspec_where {
                pub fn new() -> Self {
                    Self::default()
                }
            }
            impl #built_impl #built #built_typ #built_where {
                pub fn builder() -> #name #unspec_typ {
                    #name::new()
                }
            }
        });
        // Build fn impl
        match build_fn {
            BuildFnDef::Simple => tokens.extend(quote! {
                impl #spec_impl #name #spec_typ #spec_where {
                    pub fn build(self) -> #built #built_typ {
                        #built {
                            #(#build_assignments)*
                        }
                    }
                }
            }),
            BuildFnDef::Validating { error, body } => todo!(
                "Validating build(self) -> Result<_, {:?}> function: {:?}",
                error,
                body
            ),
            BuildFnDef::Into => tokens.extend(quote! {
                impl #spec_impl #name #spec_typ #spec_where {
                    pub fn build(self) -> #built #built_typ {
                        self.into()
                    }
                }
            }),
            BuildFnDef::TryInto => tokens.extend(quote! {
                impl #spec_impl #name #spec_typ #spec_where {
                    pub fn build(self) -> Result<#built #built_typ, <Self as TryInto<Built>>::Err> {
                        self.try_into()
                    }
                }
            }),
            BuildFnDef::Suppress => (),
        }

        // Setters impls
        for (idx, setter) in setters.iter().enumerate() {
            let prop = &setter.prop;
            let props = all_props.iter().filter(|p| *p != prop).collect::<Vec<_>>();
            let typ = &setter.typ;
            let (in_impl, in_typ, in_where) = if setter.override_allowed {
                field_generics.all_generic()
            } else {
                field_generics.for_fields(|i| if i == idx { Some(false) } else { None })
            };
            let (_, out_typ, _) =
                field_generics.for_fields(|i| if i == idx { Some(true) } else { None });
            let (set_prop, try_set_prop) = match setter.required {
                FieldRequirement::Defaulted => (quote!{ value.into() }, quote!{ value.try_into()? }),
                _ => (quote! { Some(value.into()) }, quote! { Some(value.try_into()?) }),
            };
            let fn_owned = setter.setter_ident_owned.as_ref().map(|ident| {
                quote! {
                    pub fn #ident (self, value: impl Into<#typ>) -> #name #out_typ {
                        #name {
                            #(#props,)*
                            #prop: #set_prop,
                        }
                    }
                }
            });
            let fn_borrowed = setter.setter_ident_borrowed.as_ref().map(|ident| {
                quote! {
                    pub fn #ident (&mut self, value: impl Into<#typ>) -> #name #out_typ {
                        #name {
                            #(#props,)*
                            #prop: #set_prop,
                        }
                    }
                }
            });
            let fn_try = setter.setter_ident_try.as_ref().map(|ident| quote! {
                pub fn #ident <ValueInto: TryInto<#typ>>(&mut self, value: ValueInto) -> Result<(), ValueInto::Err> {
                    Ok(#name {
                        #(#props,)*
                        #prop: #try_set_prop,
                    })
                }
            });
            if fn_owned.is_some() || fn_borrowed.is_some() || fn_try.is_some() {
                tokens.extend(quote! {
                    impl #in_impl #name #in_typ #in_where {
                        #fn_owned
                        #fn_borrowed
                        #fn_try
                    }
                });
            }
        }
    }
}

impl From<crate::options::Options> for Builder {
    fn from(value: crate::options::Options) -> Self {
        let fields = value
            .data
            .take_struct()
            .unwrap_or_else(|| abort_call_site!("Cannot derive builder for enums"));
        let derives = &*value.derive;
        let mut attrs = Vec::new();//value.attrs;
        if !derives.is_empty() {
            // TODO: use span from derives?
            match Attribute::parse_outer.parse2(quote! { #[derive(#(#derives),*)]}) {
                Ok(att) => attrs.extend(att),
                Err(e) => abort_call_site!("Error creating derive attribute: {}", e),
            }
        }
        let owned = value
            .generation_spec
            .owned
            .unwrap_or(FnGenerationSpec::PropName);
        let borrowed = value
            .generation_spec
            .referenced
            .unwrap_or(FnGenerationSpec::Format("set_{}".into()));
        let maybe = value
            .generation_spec
            .maybe
            .unwrap_or(FnGenerationSpec::Format("try_set_{}".into()));
        Self {
            attrs,
            vis: value.visibility.unwrap_or(value.vis),
            name: value
                .name
                .unwrap_or_else(|| format_ident!("{}Builder", &value.ident)),
            built: value.ident,
            generics_inherit: value.generics,
            all_props: fields.iter().map(|f| f.ident.clone()).collect(),
            setters: fields
                .into_iter()
                .map(|f| {
                    let prop = f.ident;
                    let setter_ident_owned = f
                        .functions
                        .owned
                        .as_ref()
                        .unwrap_or(&owned)
                        .make_ident(&prop);
                    let setter_ident_borrowed = f
                        .functions
                        .referenced
                        .as_ref()
                        .unwrap_or(&borrowed)
                        .make_ident(&prop);
                    let setter_ident_try = f
                        .functions
                        .maybe
                        .as_ref()
                        .unwrap_or(&maybe)
                        .make_ident(&prop);
                    FieldSetterDef {
                        prop,
                        typ: f.typ,
                        required: f.required,
                        override_allowed: f.allow_override,
                        setter_ident_owned,
                        setter_ident_borrowed,
                        setter_ident_try,
                    }
                })
                .collect(),
            build_fn: value.build,
        }
    }
}
