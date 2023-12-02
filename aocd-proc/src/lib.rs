use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Expr, LitInt, Token};

struct ClientArgs {
    year: u16,
    day: u8,
    test_input_file: Option<String>,
}

impl Parse for ClientArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let help_text = format!(
            "Provide a year and a day, e.g. #[aocd({})]",
            chrono::Utc::now().format("%Y, %d")
        );

        let year = input
            .parse::<LitInt>()
            .unwrap_or_else(|_| panic!("Expected a literal year. {help_text}"))
            .base10_parse::<u16>()?;
        input
            .parse::<Token![,]>()
            .unwrap_or_else(|_| panic!("Expected 2 arguments. {help_text}"));
        let day = input
            .parse::<LitInt>()
            .unwrap_or_else(|_| panic!("Expected a literal day. {help_text}"))
            .base10_parse::<u8>()?;

        let mut test_input_file = None;
        if input.parse::<Token![,]>().is_ok() {
            if let Ok(file_name) = input.parse::<syn::LitStr>() {
                assert!(
                    std::fs::metadata(file_name.value()).is_ok(),
                    "Test file {} does not exist",
                    file_name.value()
                );
                test_input_file = Some(file_name.value());
            }
        }

        Ok(ClientArgs {
            year,
            day,
            test_input_file,
        })
    }
}

struct SubmitArgs {
    part: Expr,
    answer: Expr,
}

impl Parse for SubmitArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let part = input.parse::<Expr>()?;

        // If the expr is a literal integer, give an error if it isn't 1 or 2.
        if let Expr::Lit(part_lit) = &part {
            if let syn::Lit::Int(part_int) = &part_lit.lit {
                if let Ok(part) = part_int.base10_parse::<i64>() {
                    assert!(part == 1 || part == 2, "Part should be 1 or 2, not {part}",);
                }
            }
        }

        input.parse::<Token![,]>()?;
        let answer: Expr = input.parse()?;
        Ok(SubmitArgs { part, answer })
    }
}

/// Annotate your main function with `#[aocd(year, day)]`.
///
/// This sets up your main function so that you can use the `aocd::input!` and `aocd::submit!` macros.
///
/// You can optionally provide a third argument, a file name. If you do, this is treated as a
/// test-input, containing a smaller input that you want to test your code on before submitting.
/// In this case, the `aocd::input!` macro will read the input from that file instead of fetching
/// it from the website, and the `aocd::submit!` macro will just be a println alias.
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
///
/// ```ignore
/// use aocd::prelude::*;  // Same as `use aocd::*;', but clippy allows it.
///
/// #[aocd(2015, 1, "test_input.txt")]
/// fn main() {
///    let part_1_answer = input!().lines().len();  // Reads from test_input.txt
///    submit!(1, part_1_answer);  // Just prints the answer, doesn't submit it.
/// }
/// ```
///
/// # Panics
/// Panics (i.e. surfaces a compile error) if the arguments are not two integers in the expected ranges,
/// or if the optional third argument is not a string literal containing a valid file name.
#[proc_macro_attribute]
pub fn aocd(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ClientArgs);
    let year = args.year;
    let day = args.day;
    let test_input_file = args.test_input_file;

    // When https://github.com/rust-lang/rust/issues/54140 is closed, use that to get nicer error messages.
    assert!(
        year >= 2015,
        "The first Advent of Code was in 2015, not {year}.",
    );
    assert!(
        (1..=25).contains(&day),
        "Chose a day from 1 to 25, not {day}.",
    );

    let mut fn_item: syn::ItemFn = syn::parse(input).unwrap();
    if let Some(test_input_file) = test_input_file {
        fn_item.block.stmts.insert(
            0,
            syn::parse(
                quote!( let __aocd_client = aocd::Aocd::new(#year, #day, Some(#test_input_file));)
                    .into(),
            )
            .unwrap(),
        );
    } else {
        fn_item.block.stmts.insert(
            0,
            syn::parse(quote!( let __aocd_client = aocd::Aocd::new(#year, #day, None);).into())
                .unwrap(),
        );
    }

    TokenStream::from(quote!(#fn_item))
}

/// Returns the puzzle input as a String: `input!()`.
///
/// This must be used within a function annotated with `#[aocd(year, day)]`.
///
/// If you provide a file name in the function annotation, it will read the input from that file instead of fetching it from the website.
/// This can be useful for testing with a smaller input, like the example input given in the puzzle description.
#[proc_macro]
pub fn input(_: TokenStream) -> TokenStream {
    TokenStream::from(quote!(__aocd_client.get_input()))
}

/// Submit an answer for the given part: `submit!(part, answer)`.
///
/// This must be used within a function annotated with `#[aocd(year, day)]`.
///
/// If you provide a file name in the function annotation, this just prints the answer without
/// submitting it to Advent of Code.
#[proc_macro]
pub fn submit(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as SubmitArgs);
    let part = args.part;
    let answer = args.answer;
    TokenStream::from(quote!(__aocd_client.submit(#part, #answer)))
}
