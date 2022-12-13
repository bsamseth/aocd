use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Expr, LitInt, Token};

struct ClientArgs {
    year: u16,
    day: u8,
}

impl Parse for ClientArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let vars = Punctuated::<LitInt, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect::<Vec<_>>();

        assert!(
            vars.len() == 2,
            "Expected 2 arguments, got {}. Provide a year and a day, e.g. #[aocd({})]",
            vars.len(),
            chrono::Utc::now().format("%Y, %d")
        );
        Ok(ClientArgs {
            year: vars[0].clone().base10_parse::<u16>()?,
            day: vars[1].clone().base10_parse::<u8>()?,
        })
    }
}

struct SubmitArgs {
    part: u8,
    answer: Expr,
}

impl Parse for SubmitArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let part: u8 = input.parse::<LitInt>()?.base10_parse::<u8>()?;
        assert!(part == 1 || part == 2, "Part should be 1 or 2, no {}", part);
        input.parse::<Token![,]>()?;
        let answer: Expr = input.parse()?;
        Ok(SubmitArgs { part, answer })
    }
}

/// Annotate your main function with `#[aocd(year, day)]`.
///
/// This sets up your main function so that you can use the `aocd::input!` and `aocd::submit!` macros.
///
/// # Example
/// ```ignore
/// use aocd::*;
///
/// #[aocd(2015, 1)]
/// fn main() {
///    let part_1_answer = input!().lines().len();
///    submit!(1, part_1_answer);
/// }
/// ```
#[proc_macro_attribute]
pub fn aocd(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ClientArgs);
    let year = args.year;
    let day = args.day;

    // When https://github.com/rust-lang/rust/issues/54140 is closed, use that to get nicer error messages.
    assert!(
        year >= 2015,
        "The first Advent of Code was in 2015, not {}.",
        year
    );
    assert!(
        day >= 1 && day <= 25,
        "Chose a day from 1 to 25, not {}.",
        day
    );

    let mut fn_item: syn::ItemFn = syn::parse(input).unwrap();
    fn_item.block.stmts.insert(
        0,
        syn::parse(quote!(let __aocd_client = aocd::Aocd::new(#year, #day);).into()).unwrap(),
    );

    TokenStream::from(quote!(#fn_item))
}

/// Returns the puzzle input as a String: `input!()`.
///
/// This must be used within a function annotated with `#[aocd(year, day)]`.
#[proc_macro]
pub fn input(_: TokenStream) -> TokenStream {
    TokenStream::from(quote!(__aocd_client.get_input()))
}

/// Submit an answer for the given part: `submit!(part, answer)`.
///
/// This must be used within a function annotated with `#[aocd(year, day)]`.
#[proc_macro]
pub fn submit(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as SubmitArgs);
    let part = args.part;
    let answer = args.answer;
    TokenStream::from(quote!(__aocd_client.submit(#part, #answer)))
}
