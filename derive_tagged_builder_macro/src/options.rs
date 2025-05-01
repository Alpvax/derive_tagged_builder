use darling::util::PathList;
use darling::FromMeta;
use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::{IdentFragment, ToTokens};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{Field, Meta, MetaList, Type};
use syn::{Generics, Ident};

#[derive(Debug, Clone)]
pub(crate) enum FnGenerationSpec {
    /// Do not generate function
    Disabled,
    /// Generate function with the same name as the property
    PropName,
    /// Generate function with the specified name.
    Fixed(Ident),
    /// Generate function with the name defined by the format string.
    Format(ecow::EcoString),
}
#[allow(dead_code)] //XXX
pub(crate) trait NameFormatter<'a> {
    fn make_name_tokens<'s: 'a, 'p>(&'s self, prop: &'p Ident) -> TokenStream {
        self.make_name(prop).into_token_stream()
    }
    fn make_name<'s: 'a, 'p>(&'s self, prop: &'p Ident) -> Option<Ident>;
}
impl<'a, F> NameFormatter<'a> for &'a F
where
    F: 'a + for<'p> Fn(&'p Ident) -> Option<Ident>,
{
    fn make_name<'s: 'a, 'p>(&'s self, prop: &'p Ident) -> Option<Ident> {
        self(prop)
    }
}
impl<'a> NameFormatter<'a> for fn(&Ident) -> Ident {
    fn make_name<'s: 'a, 'p>(&'s self, prop: &'p Ident) -> Option<Ident> {
        Some(self(prop))
    }
}
impl<'a> NameFormatter<'a> for Option<()> {
    fn make_name<'s: 'a, 'p>(&'s self, _prop: &'p Ident) -> Option<Ident> {
        None
    }
}
struct IdentFmt<'p>(&'p Ident);
impl runtime_format::FormatKey for IdentFmt<'_> {
    fn fmt(
        &self,
        key: &str,
        f: &mut core::fmt::Formatter<'_>,
    ) -> Result<(), runtime_format::FormatKeyError> {
        // #[cfg(feature = "")]
        if self.0.fmt(f).is_err() {
            abort_call_site!(
                "Error formatting IdentFmt({}) as FormatKey with \"{key}\"",
                self.0
            );
        }
        Ok(())
        // match key {
        //     "prop" => self.0.fmt(f).map_err(runtime_format::FormatKeyError::Fmt),
        //     _ => Err(runtime_format::FormatKeyError::UnknownKey),
        // }
    }
}
impl<'a> NameFormatter<'a> for &'a Ident {
    fn make_name<'s: 'a, 'p>(&'s self, _prop: &'p Ident) -> Option<Ident> {
        Some(self.to_owned().clone())
    }
    fn make_name_tokens<'s: 'a, 'p>(&'s self, _prop: &'p Ident) -> TokenStream {
        self.to_token_stream()
    }
}
impl<'a> NameFormatter<'a> for runtime_format::ParsedFmt<'a> {
    fn make_name<'s: 'a, 'p>(&'s self, prop: &'p Ident) -> Option<Ident> {
        let name = self.with_args(&IdentFmt(prop)).to_string();
        Some(Ident::new(&name, prop.span()))
    }
}
impl FnGenerationSpec {
    #[allow(dead_code)] //XXX
    pub(crate) fn ident_builder(&self) -> Box<dyn NameFormatter + '_> {
        match self {
            FnGenerationSpec::Disabled => Box::new(None),
            FnGenerationSpec::PropName => Box::new(&|p: &Ident| Some(p.clone())),
            FnGenerationSpec::Fixed(ident) => Box::new(ident),
            FnGenerationSpec::Format(fmt_str) => match runtime_format::ParsedFmt::new(fmt_str) {
                Ok(fmt) => Box::new(fmt),
                Err(_) => Box::new(None),
            },
        }
    }
    pub(crate) fn make_ident(&self, prop: &Ident) -> Option<Ident> {
        match self {
            FnGenerationSpec::Disabled => None,
            FnGenerationSpec::PropName => Some(prop.clone()),
            FnGenerationSpec::Fixed(ident) => Some(ident.clone()),
            FnGenerationSpec::Format(fmt_str) => Some(Ident::new(
                &{
                    let id = IdentFmt(prop);
                    let fmt = runtime_format::FormatArgs::<str, IdentFmt>::new(fmt_str, &id);
                    if let Err(e) = fmt.status() {
                        let err = format!("Error formatting ident at runtime!: {e}");
                        eprintln!("{err}");
                        err
                    } else {
                        format!("{}", fmt)
                    }
                },
                prop.span(),
            )),
        }
    }
    fn from_string_format(s: &str) -> Result<Self, runtime_format::FormatError> {
        runtime_format::ParsedFmt::new(s)?;
        Ok(Self::Format(s.into()))
    }
}
impl FromMeta for FnGenerationSpec {
    fn from_bool(value: bool) -> darling::Result<Self> {
        Ok(if value {
            Self::PropName
        } else {
            Self::Disabled
        })
    }
    fn from_string(value: &str) -> darling::Result<Self> {
        Self::from_string_format(value).map_err(darling::Error::custom)
    }
}
#[derive(Debug, Clone, FromMeta)]
pub(crate) struct GenerationData {
    pub(crate) owned: Option<FnGenerationSpec>,
    pub(crate) referenced: Option<FnGenerationSpec>,
    pub(crate) maybe: Option<FnGenerationSpec>,
}
impl Default for GenerationData {
    fn default() -> Self {
        Self {
            owned: Some(FnGenerationSpec::PropName),
            referenced: Some(FnGenerationSpec::Format("set_{}".into())),
            maybe: Some(FnGenerationSpec::Format("try_set_{}".into())),
        }
    }
}

#[derive(Debug, Clone, Default, FromMeta)]
pub(crate) enum BuildFnDef {
    #[default]
    /// Automatically implemented build function, setting each value from the builder
    Simple,
    /// TODO: implement Validating build -> Result<Type, error>
    Validating { error: syn::Type, body: syn::Expr },
    /// Generate build impl of Builder::into, allowing user to specify impl by implementing From<Builder> for <Type>
    Into,
    /// Generate build impl of Builder::try_into, allowing user to specify impl by implementing TryFrom<Builder> for <Type>
    TryInto,
    /// Do not generate build impl, allow user to manually implement Builder::build.
    /// Probably shouldn't often use this as it will make the majority of the derive less useful.
    /// Use Validating instead
    Suppress,
}

#[derive(Debug, Clone)]
pub(crate) enum GenericArg {
    /// <T>
    // Name(TypeParam),
    Name(Ident),
    /// <const NAME: bool>
    // Const(ConstParam),
    Const(Ident, Type),
}
impl GenericArg {
    pub(crate) fn get_ident(&self) -> &Ident {
        match self {
            GenericArg::Name(ident) => ident,
            GenericArg::Const(ident, _) => ident,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum FieldRequirement {
    /// Stored as Option<T> on builder, and struct cannot be built until the relevant setter has been called
    Required(GenericArg),
    /// Stored as T on builder, and struct can be built without calling the relevant setter
    Defaulted,
    /// Stored as Option<T> on builder, but if unspecified when build is called then the expression will be substituted
    FallbackTo(syn::Expr),
    /// Stored as Option<T> on builder, but if unspecified when build is called then the factory expression will be called
    FallbackFn(syn::Expr),
}

#[derive(Debug, Clone)]
// #[darling()]
pub(crate) struct FieldOptions {
    pub(crate) ident: Ident,
    pub(crate) typ: Type,
    pub(crate) required: FieldRequirement,
    pub(crate) allow_override: bool,
    pub(crate) functions: GenerationData,
}

struct Initialise<T>(Option<T>, Vec<darling::Error>);
impl<T> Initialise<T> {
    fn new() -> Self {
        Self(None, Vec::new())
    }
    #[allow(dead_code)] //XXX
    fn set(&mut self, value: T) {
        if self.0.is_some() {
            self.1
                .push(darling::Error::duplicate_field("optional/default/tag"));
        } else {
            self.0 = Some(value);
        }
    }
    fn set_spanned<S>(&mut self, span: &S, value: T)
    where
        S: syn::spanned::Spanned,
    {
        if self.0.is_some() {
            self.1
                .push(darling::Error::duplicate_field("optional/default/tag").with_span(span));
        } else {
            self.0 = Some(value);
        }
    }
    fn maybe_set(&mut self, value: T) {
        if self.0.is_none() {
            self.0 = Some(value);
        }
    }
}
impl darling::FromField for FieldOptions {
    fn from_field(field: &Field) -> darling::Result<Self> {
        let ident = field.ident.as_ref().ok_or(
            darling::Error::unsupported_shape_with_expected(
                darling::util::Shape::Tuple.description(),
                &darling::util::Shape::Named.description(),
            )
            .with_span(field),
        )?;
        field.attrs.iter().find_map(|att| match darling::util::parse_attribute_to_meta_list(att) {
            Ok(MetaList { path, tokens, .. }) if darling::util::path_to_string(&path) == "builder" => {
                Some(if tokens.is_empty() {
                    Err(darling::Error::missing_field("tag | optional").with_span(&tokens))
                } else {
                    match syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated.parse2(tokens) {
                        Ok(parts) => {
                            let mut errors = Vec::new();
                            let mut required = Initialise::new();
                            let mut allow_override = Initialise::new();
                            let mut owned = Initialise::new();
                            let mut borrowed = Initialise::new();
                            let mut maybe = Initialise::new();
                            for (i, meta) in parts.into_iter().enumerate() {
                                match (darling::util::path_to_string(&path).as_str(), meta) {
                                    ("optional" | "default", Meta::Path(p)) => {
                                        required.set_spanned(&p, FieldRequirement::Defaulted);
                                    },
                                    ("default", Meta::List(MetaList { tokens, .. })) => {
                                        let mut retry = true;
                                        if let Ok(c) = syn::parse2::<syn::ExprClosure>(tokens.clone()) {
                                            if let syn::ReturnType::Type(_, t) = &c.output {
                                                if c.inputs.is_empty() && t.as_ref() == &field.ty {
                                                    required.set_spanned(&c, FieldRequirement::FallbackFn(syn::Expr::Closure(c.clone())));
                                                    retry = false;
                                                }
                                            }
                                        }
                                        if retry {
                                            let span = tokens.span();
                                            match syn::parse2::<syn::Expr>(tokens) {
                                                Ok(expr) => required.set_spanned(&expr, FieldRequirement::FallbackTo(expr.clone())),
                                                Err(e) => errors.push(darling::Error::custom(format!("Error parsing default value: {e:?}")).with_span(&span)),
                                            }
                                        }
                                    },
                                    ("default", Meta::NameValue(m)) => match m.value {
                                        ref e @ syn::Expr::Closure(ref c) if c.inputs.is_empty() => match &c.output {
                                            syn::ReturnType::Type(_, t) if t.as_ref() == &field.ty => required.set_spanned(&e, FieldRequirement::FallbackFn(syn::Expr::Closure(c.clone()))),
                                            _ => required.set_spanned(&e, FieldRequirement::FallbackTo(e.clone())),
                                        }
                                        e => required.set_spanned(&e, FieldRequirement::FallbackTo(e.clone())),
                                    },
                                    ("tag", Meta::Path(_)) => errors.push(darling::Error::missing_field("tag/optional/default")),
                                    ("tag", Meta::List(MetaList { tokens, .. })) => {
                                        match syn::parse2::<syn::GenericParam>(tokens) {
                                            Ok(syn::GenericParam::Type(t)) => if t.bounds.is_empty() {
                                                required.set_spanned(&t, FieldRequirement::Required(GenericArg::Name(t.ident.clone())));
                                            } else {
                                                errors.push(darling::Error::custom("Cannot specify bounds for tag".to_string()).with_span(&t));
                                            },
                                            Ok(syn::GenericParam::Const(c)) => required.set_spanned(&c, FieldRequirement::Required(GenericArg::Const(c.ident.clone(), c.ty.clone()))),
                                            Ok(syn::GenericParam::Lifetime(lt)) => errors.push(darling::Error::custom("Cannot use a lifetime as the tag".to_string()).with_span(&lt)),
                                            Err(e) => errors.push(darling::Error::custom(format!("Failed to parse tag from tokens: {e:?}"))),
                                        }
                                    }
                                    ("tag", Meta::NameValue(m)) => {
                                        match m.value {
                                            syn::Expr::Verbatim(tokens) => match syn::parse2::<syn::GenericParam>(tokens) {
                                                Ok(syn::GenericParam::Type(t)) => if t.bounds.is_empty() {
                                                    required.set_spanned(&t, FieldRequirement::Required(GenericArg::Name(t.ident.clone())));
                                                } else {
                                                    errors.push(darling::Error::custom("Cannot specify bounds for tag".to_string()).with_span(&t));
                                                },
                                                Ok(syn::GenericParam::Const(c)) => required.set_spanned(&c, FieldRequirement::Required(GenericArg::Const(c.ident.clone(), c.ty.clone()))),
                                                Ok(syn::GenericParam::Lifetime(lt)) => errors.push(darling::Error::custom("Cannot use a lifetime as the tag".to_string()).with_span(&lt)),
                                                Err(e) => errors.push(darling::Error::custom(format!("Failed to parse tag from tokens: {e:?}"))),
                                            }
                                            syn::Expr::Path(p) => if let Some(tag) =  p.path.get_ident() {
                                                required.set_spanned(tag, FieldRequirement::Required(GenericArg::Name(tag.clone())))
                                            } else {
                                                errors.push(darling::Error::unknown_field_path(&path).with_span(&path))
                                            }
                                            e => errors.push(darling::Error::unexpected_type(&format!("Recieved an expression for the \"tag\" field! {e:?}")).with_span(&e)),
                                        }
                                    }
                                    (_, Meta::Path(path)) if i == 0 => if let Some(tag) = path.get_ident() {
                                        required.set_spanned(tag, FieldRequirement::Required(GenericArg::Name(tag.clone())))
                                    } else {
                                        errors.push(darling::Error::unknown_field_path(&path).with_span(&path))
                                    }

                                    ("allow_override" | "override" | "overrideable" | "overridable", Meta::Path(p)) => allow_override.set_spanned(&p, true),
                                    ("no_override", Meta::Path(p)) => allow_override.set_spanned(&p, false),
                                    ("allow_override" | "override" | "overrideable" | "overridable", Meta::List(m)) => match syn::parse2::<syn::LitBool>(m.tokens) {
                                        Ok(b) => allow_override.set_spanned(&b, b.value),
                                        Err(e) => errors.push(darling::Error::unknown_value(&format!("{e:?}"))),
                                    }
                                    ("allow_override" | "override" | "overrideable" | "overridable", Meta::NameValue(m)) => match m.value {
                                        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) => allow_override.set_spanned(&b, b.value),
                                        e => errors.push(darling::Error::unknown_value(&format!("{e:?}"))),
                                    }

                                    ("skip" | "ignore", Meta::Path(_)) => {
                                        owned.maybe_set(FnGenerationSpec::Disabled);
                                        borrowed.maybe_set(FnGenerationSpec::Disabled);
                                        maybe.maybe_set(FnGenerationSpec::Disabled);
                                    }
                                    ("skip" | "ignore", Meta::List(m)) => {
                                        match syn::punctuated::Punctuated::<Ident, syn::Token![,]>::parse_terminated.parse2(m.tokens) {
                                            Ok(keys) => for k in keys {
                                                match k.to_string().as_str() {
                                                    "owned" => owned.set_spanned(&k, FnGenerationSpec::Disabled),
                                                    "borrowed" | "reference" => borrowed.set_spanned(&k, FnGenerationSpec::Disabled),
                                                    "try" | "maybe" | "try_set" => maybe.set_spanned(&k, FnGenerationSpec::Disabled),
                                                    value => errors.push(darling::Error::unknown_value(value).with_span(&k)),
                                                }
                                            },
                                            Err(_) => todo!(),
                                        }
                                    }
                                    ("skip" | "ignore", Meta::NameValue(m)) => {
                                        match m.value {
                                            syn::Expr::Array(arr) => for e in arr.elems {
                                                match e {
                                                    syn::Expr::Path(p) => match darling::util::path_to_string(&p.path).as_str() {
                                                        "owned" => owned.set_spanned(&p, FnGenerationSpec::Disabled),
                                                        "borrowed" | "reference" => borrowed.set_spanned(&p, FnGenerationSpec::Disabled),
                                                        "try" | "maybe" | "try_set" => maybe.set_spanned(&p, FnGenerationSpec::Disabled),
                                                        path => errors.push(darling::Error::custom(format!("Unable to parse path as setter type [owned, borrowed, maybe | try | try_set]: {path}")).with_span(&p)),
                                                    }
                                                    syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(l), ..}) => match l.value().as_str() {
                                                        "owned" => owned.set_spanned(&l, FnGenerationSpec::Disabled),
                                                        "borrowed" | "reference" => borrowed.set_spanned(&l, FnGenerationSpec::Disabled),
                                                        "try" | "maybe" | "try_set" => maybe.set_spanned(&l, FnGenerationSpec::Disabled),
                                                        s => errors.push(darling::Error::custom(format!("Unable to parse setter type [owned, borrowed, maybe | try | try_set]: {s}")).with_span(&l)),
                                                    }
                                                    _ => errors.push(darling::Error::custom(format!("Unable to parse expr as setter type [owned, borrowed, maybe | try | try_set]: {e:?}")).with_span(&e)),
                                                }
                                            }
                                            syn::Expr::Path(p) => match darling::util::path_to_string(&p.path).as_str() {
                                                "owned" => owned.set_spanned(&p, FnGenerationSpec::Disabled),
                                                "borrowed" | "reference" => borrowed.set_spanned(&p, FnGenerationSpec::Disabled),
                                                "try" | "maybe" | "try_set" => maybe.set_spanned(&p, FnGenerationSpec::Disabled),
                                                path => errors.push(darling::Error::custom(format!("Unable to parse path as setter type [owned, borrowed, maybe | try | try_set]: {path}")).with_span(&p)),
                                            }
                                            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(l), ..}) => match l.value().as_str() {
                                                "owned" => owned.set_spanned(&l, FnGenerationSpec::Disabled),
                                                "borrowed" | "reference" => borrowed.set_spanned(&l, FnGenerationSpec::Disabled),
                                                "try" | "maybe" | "try_set" => maybe.set_spanned(&l, FnGenerationSpec::Disabled),
                                                s => errors.push(darling::Error::custom(format!("Unable to parse setter type [owned, borrowed, maybe | try | try_set]: {s}")).with_span(&l)),
                                            }
                                            e => errors.push(darling::Error::custom(format!("Unable to parse expr as setter type [owned, borrowed, maybe | try | try_set]: {e:?}")).with_span(&e)),
                                        }
                                    }
                                    ("owned", Meta::Path(p)) => owned.set_spanned(&p, FnGenerationSpec::PropName),
                                    ("borrowed" | "reference", Meta::Path(p)) => borrowed.set_spanned(&p, FnGenerationSpec::PropName),
                                    ("try" | "try_set", Meta::Path(p)) => maybe.set_spanned(&p, FnGenerationSpec::PropName),
                                    ("owned", Meta::List(m)) => {
                                        if let Ok(b) = syn::parse2::<syn::LitBool>(m.tokens.clone()) {
                                            owned.set_spanned(&m.tokens, if b.value {
                                                FnGenerationSpec::PropName
                                            } else {
                                                FnGenerationSpec::Disabled
                                            });
                                        } else if let Ok(name) = syn::parse2::<syn::Ident>(m.tokens.clone()) {
                                            owned.set_spanned(&m.tokens, FnGenerationSpec::Fixed(name));
                                        } else {
                                            let span = m.tokens.span();
                                            match syn::parse2::<syn::LitStr>(m.tokens) {
                                                Ok(s) => owned.set_spanned(&s, FnGenerationSpec::Format(s.value().into())),
                                                Err(e) => errors.push(darling::Error::custom(format!("Error parsing ident or format string: {e:?}")).with_span(&span)),
                                            }
                                        }
                                    },
                                    ("borrowed" | "reference", Meta::List(m)) => {
                                        if let Ok(b) = syn::parse2::<syn::LitBool>(m.tokens.clone()) {
                                            borrowed.set_spanned(&m.tokens, if b.value {
                                                FnGenerationSpec::PropName
                                            } else {
                                                FnGenerationSpec::Disabled
                                            });
                                        } else if let Ok(name) = syn::parse2::<syn::Ident>(m.tokens.clone()) {
                                            borrowed.set_spanned(&m.tokens, FnGenerationSpec::Fixed(name));
                                        } else {
                                            let span = m.tokens.span();
                                            match syn::parse2::<syn::LitStr>(m.tokens) {
                                                Ok(s) => borrowed.set_spanned(&s, FnGenerationSpec::Format(s.value().into())),
                                                Err(e) => errors.push(darling::Error::custom(format!("Error parsing ident or format string: {e:?}")).with_span(&span)),
                                            }
                                        }
                                    },
                                    ("try" | "try_set", Meta::List(m)) => {
                                        if let Ok(b) = syn::parse2::<syn::LitBool>(m.tokens.clone()) {
                                            maybe.set_spanned(&m.tokens, if b.value {
                                                FnGenerationSpec::PropName
                                            } else {
                                                FnGenerationSpec::Disabled
                                            });
                                        } else if let Ok(name) = syn::parse2::<syn::Ident>(m.tokens.clone()) {
                                            maybe.set_spanned(&m.tokens, FnGenerationSpec::Fixed(name));
                                        } else {
                                            let span = m.tokens.span();
                                            match syn::parse2::<syn::LitStr>(m.tokens) {
                                                Ok(s) => maybe.set_spanned(&s, FnGenerationSpec::Format(s.value().into())),
                                                Err(e) => errors.push(darling::Error::custom(format!("Error parsing ident or format string: {e:?}")).with_span(&span)),
                                            }
                                        }
                                    },
                                    ("owned", Meta::NameValue(m)) => {
                                        match m.value {
                                            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) => if b.value {
                                                owned.set_spanned(&b, FnGenerationSpec::PropName);
                                            } else {
                                                owned.set_spanned(&b, FnGenerationSpec::Disabled);
                                            }
                                            syn::Expr::Path(p) => match p.path.get_ident() {
                                                Some(n) => owned.set_spanned(&p, FnGenerationSpec::Fixed(n.clone())),
                                                None => errors.push(darling::Error::unexpected_type(&darling::util::path_to_string(&p.path)).with_span(&p)),
                                            }
                                            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => owned.set_spanned(&s, FnGenerationSpec::Format(s.value().into())),
                                            e => errors.push(darling::Error::unexpected_expr_type(&e).with_span(&e)),
                                        }
                                    },
                                    ("borrowed" | "reference", Meta::NameValue(m)) => {
                                        match m.value {
                                            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) => if b.value {
                                                borrowed.set_spanned(&b, FnGenerationSpec::PropName);
                                            } else {
                                                borrowed.set_spanned(&b, FnGenerationSpec::Disabled);
                                            }
                                            syn::Expr::Path(p) => match p.path.get_ident() {
                                                Some(n) => borrowed.set_spanned(&p, FnGenerationSpec::Fixed(n.clone())),
                                                None => errors.push(darling::Error::unexpected_type(&darling::util::path_to_string(&p.path)).with_span(&p)),
                                            }
                                            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => borrowed.set_spanned(&s, FnGenerationSpec::Format(s.value().into())),
                                            e => errors.push(darling::Error::unexpected_expr_type(&e).with_span(&e)),
                                        }
                                    },
                                    ("try" | "try_set", Meta::NameValue(m)) => {
                                        match m.value {
                                            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) => if b.value {
                                                maybe.set_spanned(&b, FnGenerationSpec::PropName);
                                            } else {
                                                maybe.set_spanned(&b, FnGenerationSpec::Disabled);
                                            }
                                            syn::Expr::Path(p) => match p.path.get_ident() {
                                                Some(n) => maybe.set_spanned(&p, FnGenerationSpec::Fixed(n.clone())),
                                                None => errors.push(darling::Error::unexpected_type(&darling::util::path_to_string(&p.path)).with_span(&p)),
                                            }
                                            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => maybe.set_spanned(&s, FnGenerationSpec::Format(s.value().into())),
                                            e => errors.push(darling::Error::unexpected_expr_type(&e).with_span(&e)),
                                        }
                                    },

                                    (s, m) => errors.push(darling::Error::unknown_field(s).with_span(&m)),
                                }
                            }
                            errors.extend(required.1);
                            errors.extend(allow_override.1);
                            errors.extend(owned.1);
                            errors.extend(borrowed.1);
                            errors.extend(maybe.1);
                            let required = if let Some(r) = required.0 {
                                r
                            } else {
                                errors.push(darling::Error::missing_field("tag/optional/default"));
                                // return this to prevent panic
                                FieldRequirement::Defaulted
                            };
                            let (prop_count, fixed_names) = [owned.0.as_ref(), borrowed.0.as_ref(), maybe.0.as_ref()].into_iter().fold((0u8, std::collections::HashMap::new()), |(prop_count, mut fixed_names), spec| {
                                if let Some(FnGenerationSpec::Fixed(name)) = &spec {
                                    *fixed_names.entry(name.to_string()).or_insert(0u8) += 1;
                                }
                                (prop_count + matches!(spec, Some(FnGenerationSpec::PropName)) as u8, fixed_names)
                            });
                            if prop_count > 0 {
                                errors.push(darling::Error::custom("Cannot create setter definition where multiple setters are using the property name"));
                            }
                            errors.extend(fixed_names.into_iter().filter_map(|(name, count)| if count > 1 {
                                Some(darling::Error::custom(format!("Duplicate setter name detected: {name}")))
                            } else {
                                None
                            }));
                            if !errors.is_empty() {
                                Err(darling::Error::multiple(errors))
                            } else {
                                Ok(Self {
                                    ident: ident.clone(),
                                    typ: field.ty.clone(),
                                    required,
                                    allow_override: allow_override.0.unwrap_or(false),
                                    functions: GenerationData { owned: owned.0, referenced: borrowed.0, maybe: maybe.0 }
                                })
                            }
                        },
                        Err(e) => Err(darling::Error::custom(format!("Unable to parse \"builder\" attribute contents: {e:?}"))),
                    }
                })
            },
            _ => None,
        }).unwrap_or(Err(darling::Error::custom("Missing \"builder\" attribute on field")))
    }
}

#[derive(Debug, Clone, darling::FromDeriveInput)]
#[darling(
    attributes(builder),
    // forward_attrs(cfg, allow, builder_struct_attr, builder_impl_attr),
    supports(struct_named)
)]
pub struct Options {
    // pub(super) attrs: Vec<Attribute>,
    pub(super) ident: Ident,

    /// The visibility of the deriving struct. Do not confuse this with `#[builder(vis = "...")]`,
    /// which is received by `Options::visibility`.
    pub(super) vis: syn::Visibility,

    pub(super) generics: Generics,

    /// The name of the generated builder. Defaults to `#{ident}Builder`.
    pub(super) name: Option<Ident>,

    #[darling(default)]
    pub(super) generation_spec: GenerationData,

    #[darling(default)]
    pub(super) build: BuildFnDef,

    /// Additional traits to derive on the builder.
    #[darling(default)]
    pub(super) derive: PathList,

    /// Desired visibility of the builder struct.
    ///
    /// Do not confuse this with `Options::vis`, which is the visibility of the deriving struct.
    pub(super) visibility: Option<syn::Visibility>,

    /// The parsed body of the derived struct.
    pub(super) data: darling::ast::Data<darling::util::Ignored, FieldOptions>,
}
