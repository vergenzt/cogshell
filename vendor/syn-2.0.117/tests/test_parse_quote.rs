#![allow(
    clippy::elidable_lifetime_names,
    clippy::needless_lifetimes,
    clippy::uninlined_format_args
)]

#[macro_use]
mod snapshot;

mod debug;

use syn::punctuated::Punctuated;
use syn::{parse_quote, Attribute, Field, Lit, Pat, Stmt, Token};


fn test_attribute() {
    let attr: Attribute = parse_quote!(#[test]);
    snapshot!(attr, @r#"
    Attribute {
        style: AttrStyle::Outer,
        meta: Meta::Path {
            segments: [
                PathSegment {
                    ident: "test",
                },
            ],
        },
    }
    "#);

    let attr: Attribute = parse_quote!(#![no_std]);
    snapshot!(attr, @r#"
    Attribute {
        style: AttrStyle::Inner,
        meta: Meta::Path {
            segments: [
                PathSegment {
                    ident: "no_std",
                },
            ],
        },
    }
    "#);
}


fn test_field() {
    let field: Field = parse_quote!(pub enabled: bool);
    snapshot!(field, @r#"
    Field {
        vis: Visibility::Public,
        ident: Some("enabled"),
        colon_token: Some,
        ty: Type::Path {
            path: Path {
                segments: [
                    PathSegment {
                        ident: "bool",
                    },
                ],
            },
        },
    }
    "#);

    let field: Field = parse_quote!(primitive::bool);
    snapshot!(field, @r#"
    Field {
        vis: Visibility::Inherited,
        ty: Type::Path {
            path: Path {
                segments: [
                    PathSegment {
                        ident: "primitive",
                    },
                    Token![::],
                    PathSegment {
                        ident: "bool",
                    },
                ],
            },
        },
    }
    "#);
}


fn test_pat() {
    let pat: Pat = parse_quote!(Some(false) | None);
    snapshot!(&pat, @r#"
    Pat::Or {
        cases: [
            Pat::TupleStruct {
                path: Path {
                    segments: [
                        PathSegment {
                            ident: "Some",
                        },
                    ],
                },
                elems: [
                    Pat::Lit(ExprLit {
                        lit: Lit::Bool {
                            value: false,
                        },
                    }),
                ],
            },
            Token![|],
            Pat::Ident {
                ident: "None",
            },
        ],
    }
    "#);

    let boxed_pat: Box<Pat> = parse_quote!(Some(false) | None);
    assert_eq!(*boxed_pat, pat);
}


fn test_punctuated() {
    let punctuated: Punctuated<Lit, Token![|]> = parse_quote!(true | true);
    snapshot!(punctuated, @r#"
    [
        Lit::Bool {
            value: true,
        },
        Token![|],
        Lit::Bool {
            value: true,
        },
    ]
    "#);

    let punctuated: Punctuated<Lit, Token![|]> = parse_quote!(true | true |);
    snapshot!(punctuated, @r#"
    [
        Lit::Bool {
            value: true,
        },
        Token![|],
        Lit::Bool {
            value: true,
        },
        Token![|],
    ]
    "#);
}


fn test_vec_stmt() {
    let stmts: Vec<Stmt> = parse_quote! {
        let _;
        true
    };
    snapshot!(stmts, @r#"
    [
        Stmt::Local {
            pat: Pat::Wild,
        },
        Stmt::Expr(
            Expr::Lit {
                lit: Lit::Bool {
                    value: true,
                },
            },
            None,
        ),
    ]
    "#);
}
