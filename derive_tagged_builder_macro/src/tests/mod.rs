use darling::{FromDeriveInput, FromField};
use expect_test::{expect, expect_file};
use quote::{quote, ToTokens};

macro_rules! make_input {
    (@tokens) => {
        quote! {
            #[derive(TaggedBuilder)]
            #[builder(derive(Debug))]
            struct Foo {
                #[builder(Bar)]
                bar: bool,
                #[builder(Baz)]
                baz: Vec<u8>,
            }
        }
    };
    (@derive) => {
        match syn::parse2::<syn::DeriveInput>(make_input!(@tokens)) {
            Ok(syntax_tree) => syntax_tree,
            Err(err) => panic!("{:?}", err.to_compile_error()),
        }
    };
    () => { make_input!(@derive) };
}

#[test]
fn derive_input() {
    let ast = make_input!(@derive);
    expect![[r#"
        DeriveInput {
            attrs: [
                Attribute {
                    pound_token: Pound,
                    style: AttrStyle::Outer,
                    bracket_token: Bracket,
                    meta: Meta::List {
                        path: Path {
                            leading_colon: None,
                            segments: [
                                PathSegment {
                                    ident: Ident(
                                        derive,
                                    ),
                                    arguments: PathArguments::None,
                                },
                            ],
                        },
                        delimiter: MacroDelimiter::Paren(
                            Paren,
                        ),
                        tokens: TokenStream [
                            Ident {
                                sym: TaggedBuilder,
                            },
                        ],
                    },
                },
                Attribute {
                    pound_token: Pound,
                    style: AttrStyle::Outer,
                    bracket_token: Bracket,
                    meta: Meta::List {
                        path: Path {
                            leading_colon: None,
                            segments: [
                                PathSegment {
                                    ident: Ident(
                                        builder,
                                    ),
                                    arguments: PathArguments::None,
                                },
                            ],
                        },
                        delimiter: MacroDelimiter::Paren(
                            Paren,
                        ),
                        tokens: TokenStream [
                            Ident {
                                sym: derive,
                            },
                            Group {
                                delimiter: Parenthesis,
                                stream: TokenStream [
                                    Ident {
                                        sym: Debug,
                                    },
                                ],
                            },
                        ],
                    },
                },
            ],
            vis: Visibility::Inherited,
            ident: Ident(
                Foo,
            ),
            generics: Generics {
                lt_token: None,
                params: [],
                gt_token: None,
                where_clause: None,
            },
            data: Data::Struct {
                struct_token: Struct,
                fields: Fields::Named {
                    brace_token: Brace,
                    named: [
                        Field {
                            attrs: [
                                Attribute {
                                    pound_token: Pound,
                                    style: AttrStyle::Outer,
                                    bracket_token: Bracket,
                                    meta: Meta::List {
                                        path: Path {
                                            leading_colon: None,
                                            segments: [
                                                PathSegment {
                                                    ident: Ident(
                                                        builder,
                                                    ),
                                                    arguments: PathArguments::None,
                                                },
                                            ],
                                        },
                                        delimiter: MacroDelimiter::Paren(
                                            Paren,
                                        ),
                                        tokens: TokenStream [
                                            Ident {
                                                sym: Bar,
                                            },
                                        ],
                                    },
                                },
                            ],
                            vis: Visibility::Inherited,
                            mutability: FieldMutability::None,
                            ident: Some(
                                Ident(
                                    bar,
                                ),
                            ),
                            colon_token: Some(
                                Colon,
                            ),
                            ty: Type::Path {
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: [
                                        PathSegment {
                                            ident: Ident(
                                                bool,
                                            ),
                                            arguments: PathArguments::None,
                                        },
                                    ],
                                },
                            },
                        },
                        Comma,
                        Field {
                            attrs: [
                                Attribute {
                                    pound_token: Pound,
                                    style: AttrStyle::Outer,
                                    bracket_token: Bracket,
                                    meta: Meta::List {
                                        path: Path {
                                            leading_colon: None,
                                            segments: [
                                                PathSegment {
                                                    ident: Ident(
                                                        builder,
                                                    ),
                                                    arguments: PathArguments::None,
                                                },
                                            ],
                                        },
                                        delimiter: MacroDelimiter::Paren(
                                            Paren,
                                        ),
                                        tokens: TokenStream [
                                            Ident {
                                                sym: Baz,
                                            },
                                        ],
                                    },
                                },
                            ],
                            vis: Visibility::Inherited,
                            mutability: FieldMutability::None,
                            ident: Some(
                                Ident(
                                    baz,
                                ),
                            ),
                            colon_token: Some(
                                Colon,
                            ),
                            ty: Type::Path {
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: [
                                        PathSegment {
                                            ident: Ident(
                                                Vec,
                                            ),
                                            arguments: PathArguments::AngleBracketed {
                                                colon2_token: None,
                                                lt_token: Lt,
                                                args: [
                                                    GenericArgument::Type(
                                                        Type::Path {
                                                            qself: None,
                                                            path: Path {
                                                                leading_colon: None,
                                                                segments: [
                                                                    PathSegment {
                                                                        ident: Ident(
                                                                            u8,
                                                                        ),
                                                                        arguments: PathArguments::None,
                                                                    },
                                                                ],
                                                            },
                                                        },
                                                    ),
                                                ],
                                                gt_token: Gt,
                                            },
                                        },
                                    ],
                                },
                            },
                        },
                        Comma,
                    ],
                },
                semi_token: None,
            },
        }
    "#]].assert_debug_eq(&ast);
}


#[test]
fn parse_fields() {
    let fields: syn::FieldsNamed = syn::parse_quote! {{
        #[builder(Bar)]
        bar: bool,
        #[builder(Baz)]
        baz: Option<u8>,
    }};
    expect![[r#"
        [
            FieldOptions {
                ident: Ident(
                    bar,
                ),
                typ: Type::Path {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: [
                            PathSegment {
                                ident: Ident(
                                    bool,
                                ),
                                arguments: PathArguments::None,
                            },
                        ],
                    },
                },
                required: Required(
                    Name(
                        Ident(
                            Bar,
                        ),
                    ),
                ),
                allow_override: false,
                functions: GenerationData {
                    owned: None,
                    referenced: None,
                    maybe: None,
                },
            },
            FieldOptions {
                ident: Ident(
                    baz,
                ),
                typ: Type::Path {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: [
                            PathSegment {
                                ident: Ident(
                                    Option,
                                ),
                                arguments: PathArguments::AngleBracketed {
                                    colon2_token: None,
                                    lt_token: Lt,
                                    args: [
                                        GenericArgument::Type(
                                            Type::Path {
                                                qself: None,
                                                path: Path {
                                                    leading_colon: None,
                                                    segments: [
                                                        PathSegment {
                                                            ident: Ident(
                                                                u8,
                                                            ),
                                                            arguments: PathArguments::None,
                                                        },
                                                    ],
                                                },
                                            },
                                        ),
                                    ],
                                    gt_token: Gt,
                                },
                            },
                        ],
                    },
                },
                required: Required(
                    Name(
                        Ident(
                            Baz,
                        ),
                    ),
                ),
                allow_override: false,
                functions: GenerationData {
                    owned: None,
                    referenced: None,
                    maybe: None,
                },
            },
        ]
    "#]].assert_debug_eq(&darling::ast::Fields::<super::options::FieldOptions>::try_from(&syn::Fields::Named(fields)).expect("error converting fields").fields);
}
#[test]
fn parse_field_attr() {
    let field: syn::Field = syn::parse_quote! {
        #[builder(Bar)]
        bar: bool
    };
    expect![[r#"
        Ok(
            FieldOptions {
                ident: Ident(
                    bar,
                ),
                typ: Type::Path {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: [
                            PathSegment {
                                ident: Ident(
                                    bool,
                                ),
                                arguments: PathArguments::None,
                            },
                        ],
                    },
                },
                required: Required(
                    Name(
                        Ident(
                            Bar,
                        ),
                    ),
                ),
                allow_override: false,
                functions: GenerationData {
                    owned: None,
                    referenced: None,
                    maybe: None,
                },
            },
        )
    "#]].assert_debug_eq(&super::options::FieldOptions::from_field(&field));
}

#[test]
fn parse_options() {
    let ast = make_input!(@derive);
    expect![[r#"
        Options {
            ident: Ident(
                Foo,
            ),
            vis: Visibility::Inherited,
            generics: Generics {
                lt_token: None,
                params: [],
                gt_token: None,
                where_clause: None,
            },
            name: None,
            generation_spec: GenerationData {
                owned: Some(
                    PropName,
                ),
                referenced: Some(
                    Format(
                        "set_{}",
                    ),
                ),
                maybe: Some(
                    Format(
                        "try_set_{}",
                    ),
                ),
            },
            build: Simple,
            derive: PathList(
                [
                    Path {
                        leading_colon: None,
                        segments: [
                            PathSegment {
                                ident: Ident(
                                    Debug,
                                ),
                                arguments: PathArguments::None,
                            },
                        ],
                    },
                ],
            ),
            visibility: None,
            data: Struct(
                Fields {
                    style: Struct,
                    fields: [
                        FieldOptions {
                            ident: Ident(
                                bar,
                            ),
                            typ: Type::Path {
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: [
                                        PathSegment {
                                            ident: Ident(
                                                bool,
                                            ),
                                            arguments: PathArguments::None,
                                        },
                                    ],
                                },
                            },
                            required: Required(
                                Name(
                                    Ident(
                                        Bar,
                                    ),
                                ),
                            ),
                            allow_override: false,
                            functions: GenerationData {
                                owned: None,
                                referenced: None,
                                maybe: None,
                            },
                        },
                        FieldOptions {
                            ident: Ident(
                                baz,
                            ),
                            typ: Type::Path {
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: [
                                        PathSegment {
                                            ident: Ident(
                                                Vec,
                                            ),
                                            arguments: PathArguments::AngleBracketed {
                                                colon2_token: None,
                                                lt_token: Lt,
                                                args: [
                                                    GenericArgument::Type(
                                                        Type::Path {
                                                            qself: None,
                                                            path: Path {
                                                                leading_colon: None,
                                                                segments: [
                                                                    PathSegment {
                                                                        ident: Ident(
                                                                            u8,
                                                                        ),
                                                                        arguments: PathArguments::None,
                                                                    },
                                                                ],
                                                            },
                                                        },
                                                    ),
                                                ],
                                                gt_token: Gt,
                                            },
                                        },
                                    ],
                                },
                            },
                            required: Required(
                                Name(
                                    Ident(
                                        Baz,
                                    ),
                                ),
                            ),
                            allow_override: false,
                            functions: GenerationData {
                                owned: None,
                                referenced: None,
                                maybe: None,
                            },
                        },
                    ],
                    span: Some(
                        Span,
                    ),
                    __nonexhaustive: (),
                },
            ),
        }
    "#]].assert_debug_eq(&super::options::Options::from_derive_input(&ast).expect("Options ok"));
}

#[test]
fn parse_builder() {
    let ast = make_input!(@derive);
    let opts = super::options::Options::from_derive_input(&ast).expect("Options ok");
    expect![[r#"
        Builder {
            attrs: [
                Attribute {
                    pound_token: Pound,
                    style: AttrStyle::Outer,
                    bracket_token: Bracket,
                    meta: Meta::List {
                        path: Path {
                            leading_colon: None,
                            segments: [
                                PathSegment {
                                    ident: Ident(
                                        derive,
                                    ),
                                    arguments: PathArguments::None,
                                },
                            ],
                        },
                        delimiter: MacroDelimiter::Paren(
                            Paren,
                        ),
                        tokens: TokenStream [
                            Ident {
                                sym: Debug,
                            },
                        ],
                    },
                },
            ],
            vis: Visibility::Inherited,
            name: Ident(
                FooBuilder,
            ),
            built: Ident(
                Foo,
            ),
            generics_inherit: Generics {
                lt_token: None,
                params: [],
                gt_token: None,
                where_clause: None,
            },
            all_props: [
                Ident(
                    bar,
                ),
                Ident(
                    baz,
                ),
            ],
            setters: [
                FieldSetterDef {
                    prop: Ident(
                        bar,
                    ),
                    typ: Type::Path {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: [
                                PathSegment {
                                    ident: Ident(
                                        bool,
                                    ),
                                    arguments: PathArguments::None,
                                },
                            ],
                        },
                    },
                    required: Required(
                        Name(
                            Ident(
                                Bar,
                            ),
                        ),
                    ),
                    override_allowed: false,
                    setter_ident_owned: Some(
                        Ident(
                            bar,
                        ),
                    ),
                    setter_ident_borrowed: Some(
                        Ident(
                            set_bar,
                        ),
                    ),
                    setter_ident_try: Some(
                        Ident(
                            try_set_bar,
                        ),
                    ),
                },
                FieldSetterDef {
                    prop: Ident(
                        baz,
                    ),
                    typ: Type::Path {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: [
                                PathSegment {
                                    ident: Ident(
                                        Vec,
                                    ),
                                    arguments: PathArguments::AngleBracketed {
                                        colon2_token: None,
                                        lt_token: Lt,
                                        args: [
                                            GenericArgument::Type(
                                                Type::Path {
                                                    qself: None,
                                                    path: Path {
                                                        leading_colon: None,
                                                        segments: [
                                                            PathSegment {
                                                                ident: Ident(
                                                                    u8,
                                                                ),
                                                                arguments: PathArguments::None,
                                                            },
                                                        ],
                                                    },
                                                },
                                            ),
                                        ],
                                        gt_token: Gt,
                                    },
                                },
                            ],
                        },
                    },
                    required: Required(
                        Name(
                            Ident(
                                Baz,
                            ),
                        ),
                    ),
                    override_allowed: false,
                    setter_ident_owned: Some(
                        Ident(
                            baz,
                        ),
                    ),
                    setter_ident_borrowed: Some(
                        Ident(
                            set_baz,
                        ),
                    ),
                    setter_ident_try: Some(
                        Ident(
                            try_set_baz,
                        ),
                    ),
                },
            ],
            build_fn: Simple,
        }
    "#]].assert_debug_eq(&super::builder::Builder::from(opts));
}

#[test]
fn builder_tokens() {
    let builder = super::builder::Builder::from(super::options::Options::from_derive_input(&make_input!(@derive)).expect("Options ok"));
    expect_file!["./output.rs"].assert_eq(&prettyplease::unparse(&syn::parse2(builder.into_token_stream()).expect("Error parsing output as file contents")));
}

