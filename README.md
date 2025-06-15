# 🧪 `spipe` – Smart Pipe Macro for Rust

`spipe` is a procedural macro that brings smart pipe operators into Rust. It allows 
you to write expressive and readable transformation pipelines, inspired by 
functional languages like Elixir or F#, but tailored to Rust’s type system and 
ergonomics.

With `spipe`, you can:
- [x] Chain function and method calls with clean, readable syntax
- [x] Automatically map, unwrap, or and_then values
- [x] Inject debug logs or side effects without breaking the chain
- [x] Clone, mutate, or transform data in-place
- [x] Say goodbye to nested `.map(...).and_then(...)?` soup

> **Warning** This project is in an early development stage. Bug reports, feature requests, and 
contributions are welcome!


## 🚀 Why use spipe!?

Rust doesn’t have a native pipe (`|>`) operator. While method chaining works well for 
many cases, it falls short when:
- Mixing functions, methods, and closures
- Handling nested Result/Option types
- Performing temporary side effects or debugging
- Inserting values as arguments in arbitrary positions

`spipe!` solves this with flexible, intuitive operators and value-routing logic.


## ✨ Pipe Types

Each line in a `spipe!` pipeline begins with an operator. These dictate how the value 
is forwarded.

| Operator | Name     | Behavior                                                         |
|----------|----------|------------------------------------------------------------------|
| `=> `    | Basic    | Just pass the value to a function or method                      |
| `=>&`    | AndThen  | Like `.and_then(...)` on Result/Option                           |
| `=>@`    | Map      | Like `.map(...)` on Result/Option                                |
| `=>?`    | Try      | Applies `?` to propagate errors/None                             |
| `=>*`    | Unwrap   | Calls `.unwrap()`                                                |
| `=>+`    | Clone    | Clones the current value                                         |
| `=>#`    | Apply    | Performs a side effect (e.g. `println!`), returns original value |
| `=>$`    | ApplyMut | Like Apply, but passes mutable reference                         |

💡 Mnemonics:
- `&` - and_then
- `@` - mapping at the value
- `?` - try / propagate
- `*` - deref / unpack / unwrap
- `+` - clone
- `#` - debug / apply


## 🧾 Pipe Operation Syntax

Here’s how you control what happens to the piped value:

| Syntax                 | Operation                                             |
|------------------------|-------------------------------------------------------|
| `func or func()`       | Call the function with the value as the sole argument |
| `func(arg2, arg3)`     | Insert the piped value as the first argument          |
| `func(arg1, (), arg3)` | Substitute () with the piped value                    |
| `.method()`            | Call method on the value                              |
| `.method(arg2)`        | Piped value is the receiver (self)                    |
| `\|x\| x**2`           | Apply closure to the piped value                      |
| `(Type)`               | Call Type::from(value)                                |
| `(Type?)`              | Call Type::try_from(value)                            |
| `(as Type)`            | Convert using as                                      |
| `...`                  | Just pass the value as-is (NoOp)                      |



## 📦 Installation

Add to your Cargo.toml:

```toml
[dependencies]
spipe = "0.1" # Replace with latest version
```

Import the macro:

```rust
use spipe::spipe;
```


## ✅ Examples

### 🔁 Functional Transformation Chain

```rust
fn parse_number(s: &str) -> Result<i32, &'static str> {
    s.parse().map_err(|_| "not a number")
}

fn double(n: i32) -> i32 {
    n * 2
}

let input = "42";

let res = spipe!(
    input
        =>  parse_number
        =>& Ok
        =>@ double
        =>? (as f64)
        =># |v| println!("final value: {}", v)
);

assert_eq!(res, 84.0);
```


### 🔍 Debug Inline Without Breaking Flow

```rust
fn square(n: i32) -> i32 { n * n }

let result = spipe!(
    4
        =># |v| println!("initial: {}", v)
        =>  square
        =># |v| println!("squared: {}", v)
        =>  |x| x + 10
        =># |v| println!("plus 10: {}", v)
);

assert_eq!(result, 26);
```


### ✍️ Mutate Values In-Place

```rust
let mut result = String::from("hello");

spipe!(
    result
        => .to_uppercase()
        =>$ .push('!')
        =># |s| println!("Final: {}", s)
);

assert_eq!(result, "HELLO!");
```


### 🧩 Flexible Function Argument Substitution

```rust
fn wrap_with_brackets(prefix: &str, content: &str, suffix: &str) -> String {
format!("{}{}{}", prefix, content, suffix)
}

let raw = "core";

let wrapped = spipe!(
    raw
        => .to_string
        => .to_uppercase
        => .as_str
        => wrap_with_brackets("[", (), "]")
);

assert_eq!(wrapped, "[CORE]");
```


## 📚 Summary

`spipe!` helps you write cleaner, more expressive Rust code by:
- Wrapping up common `Option/Result` logic
- Unifying function, method, and closure syntax
- Removing nested, noisy chains
- Keeping logic readable and flat


## 🧪 Contributing

This project is in early development, and your input matters!
Ideas, bugs, PRs, new pipe operators — all welcome.


## 📖 License

MIT or Apache-2.0 — your choice.

