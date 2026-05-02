use proc_macro2::{Ident, Span, TokenStream};
use std::str::FromStr;
use syn::Result;

#[track_caller]
fn parse(s: &str) -> Result<Ident> {
    syn::parse2(TokenStream::from_str(s).unwrap())
}

#[track_caller]
fn new(s: &str) -> Ident {
    Ident::new(s, Span::call_site())
}


fn ident_parse() {
    parse("String").unwrap();
}


fn ident_parse_keyword() {
    parse("abstract").unwrap_err();
}


fn ident_parse_empty() {
    parse("").unwrap_err();
}


fn ident_parse_lifetime() {
    parse("'static").unwrap_err();
}


fn ident_parse_underscore() {
    parse("_").unwrap_err();
}


fn ident_parse_number() {
    parse("255").unwrap_err();
}


fn ident_parse_invalid() {
    parse("a#").unwrap_err();
}


fn ident_new() {
    new("String");
}


fn ident_new_keyword() {
    new("abstract");
}


#[should_panic(expected = "use Option<Ident>")]
fn ident_new_empty() {
    new("");
}


#[should_panic(expected = "not a valid Ident")]
fn ident_new_lifetime() {
    new("'static");
}


fn ident_new_underscore() {
    new("_");
}


#[should_panic(expected = "use Literal instead")]
fn ident_new_number() {
    new("255");
}


#[should_panic(expected = "\"a#\" is not a valid Ident")]
fn ident_new_invalid() {
    new("a#");
}
