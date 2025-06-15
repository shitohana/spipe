//! # `spipe` – Smart Pipe Macro for Rust
//!
//! `spipe` is a procedural macro that introduces smart pipe operators
//! into Rust — allowing you to write expressive and readable value transformation
//! pipelines, inspired by functional languages, but tailored for Rust’s semantics.
//!
//! With spipe, you can:
//! - Chain operations with clean syntax
//! - Automatically map, unwrap, or flat-map
//! - Clone or mutate mid-pipeline
//! - Debug or inspect without breaking the flow
//! 
//! > **Note:** This project is in early state of development. Bug reports and 
//! contributions are welcome!
//!
//! ## Pipe Types
//!
//! Each line in the pipeline begins with a smart pipe operator. These control
//! how the value is passed forward.
//!
//! | Operator | Name     | Behavior                                                         |
//! |----------|----------|------------------------------------------------------------------|
//! | `=> `      | Basic    | Just pass the value to a function or method                    |
//! | `=>&`      | AndThen  | Like .and_then(...) on Result/Option                           |
//! | `=>@`      | Map      | Like .map(...) on Result/Option                                |
//! | `=>?`      | Try      | Applies ? to propagate errors/None                             |
//! | `=>*`      | Unwrap   | Calls .unwrap()                                                |
//! | `=>+`      | Clone    | Clones the current value                                       |
//! | `=>#`      | Apply    | Performs a side effect (e.g. println!), returns original value |
//! | `=>$`      | ApplyMut | Like Apply, but passes mutable reference                       |
//!
//! #### Remember hints
//!
//! - `&` -- &(and)_then
//! - `@` -- mapping at your value
//! - `?` -- same as Rust’s try operator
//! - `*` -- dereference → unwrap
//! - `+` -- clone
//! - `#` -- “hashtag debug” → apply
//!
//! If you have better association ideas, you are welcome to open a pull request!
//!
//! ## Pipe Operations
//!
//! | Syntax               | Operation                                               |
//! |----------------------|---------------------------------------------------------|
//! | `func or func()`       | Call the function with the value as the sole argument |
//! | `func(arg2, arg3)`     | Insert the piped value as the first argument          |
//! | `func(arg1, (), arg3)` | Substitute () with the piped value                    |
//! | `.method()`            | Call method on the value                              |
//! | `.method(arg2)`        | Piped value is the receiver (self)                    |
//! | `\|x\| x**2`           | Apply closure to the piped value                      |
//! | `(Type)`               | Call Type::from(value)                                |
//! | `(Type?)`              | Call Type::try_from(value)                            |
//! | `(as Type)`            | Convert using as                                      |
//! | `...`                  | Just pass the value as-is (NoOp)                      |
//!
//! ## Examples
//!
//! ### Transform variable
//! ```
//! use spipe::spipe;
//! 
//! fn parse_number(s: &str) -> Result<i32, &'static str> {
//!     s.parse().map_err(|_| "not a number")
//! }
//! 
//! fn double(n: i32) -> i32 {
//!     n * 2
//! }
//! 
//! # fn main() -> Result<(), &'static str> {
//!  let input = "42";
//!
//!  let res = spipe!(
//!     input
//!         =>  parse_number          // Result<i32, _>
//!         =>& Ok                    // and_then(Ok)
//!         =>@ double                // map(double)
//!         =>? (as f64)              // convert to f64
//!         =># |s| println!("{}", s) // debug print
//!  );
//!  assert_eq!(84.0, res);
//!
//!  // Without pipe operator it would look like:
//!  // let res = parse_number(input)
//!  //    .and_then(Ok)
//!  //    .map(double)?
//!  //    as f64;
//!         
//! #   Ok(())
//! # }
//! ```
//! 
//! ### Add debug messages
//! 
//! ```
//! use spipe::spipe;
//! 
//! fn square(n: i32) -> i32 {
//!     n * n
//! }
//!
//!  let result = spipe!(
//!     4
//!         =># |v| println!("initial: {}", v)
//!         =>  square
//!         =># |v| println!("squared: {}", v)
//!         =>  |x| x + 10
//!         =># |v| println!("plus 10: {}", v)
//!  );
//!  assert_eq!(result, 26) 
//! 
//!  // Without pipe operator it would look like:
//!  // let initial = 4;
//!  // println!("initial: {}", initial);
//!  // let squared = square(initial);
//!  // println!("squared: {}", squared);
//!  // let sum = squared + 10;
//!  // println!("sum: {}", sum);
//! ```
//! 
//! ### Modify inplace
//! 
//! ```
//! use spipe::spipe;
//! 
//! let result = String::from("hello");
//!     
//!  spipe!(
//!  result
//!  => .to_uppercase()
//!  =>$ .push('!')
//!  =># |s| println!("Final string: {}", s)
//!  );
//!  
//!  assert_eq!(result, "HELLO!")
//!
//!  // Without pipe operator it would look like:
//!  // let mut uppercased = result.to_uppercase();
//!  // uppercased.push( '!');
//!  // println!("{:?}", uppercased);
//! ```
//!
//! ### Substitution
//! 
//! ```
//! use spipe::spipe;
//!
//! fn wrap_with_brackets(prefix: &str, content: &str, suffix: &str) -> String {
//!     format!("{}{}{}", prefix, content, suffix)
//! }
//! 
//! let raw = "core";
//!
//! let wrapped = spipe!(
//!     raw 
//!         => .to_string
//!         => .to_uppercase
//!         => .as_str
//!         => wrap_with_brackets("[", (), "]") // Insert raw in ()
//! );
//! 
//! // Without pipe operator it would look like:
//! // let wrapped = wrap_with_brackets(
//! //     "[", 
//! //     raw.to_string().to_uppercase().as_str(), 
//! //     "]"
//! // );
//!
//! assert_eq!(wrapped, "[CORE]");
//! ```
//! 
//! `spipe!` helps you write cleaner, more expressive Rust pipelines by choosing
//! the right transformation based on your intent:
//! - Use `=>@`, `=>&`, `=>?` for functional types (Result, Option)
//! - Use closures or substitutions for more flexible function calls
//! - Insert debug logs or mutations inline
//! - Reduce boilerplate and increase readability

extern crate proc_macro;
pub(crate) mod pipe;
pub(crate) mod utils;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::pipe::MacroInput;

#[proc_macro]
pub fn spipe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MacroInput);

    match input.run() {
        Ok(expr) => quote::quote! { #expr }.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
