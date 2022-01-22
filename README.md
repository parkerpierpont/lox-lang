## Lox Implementation

A super-naive implementation of Lox from the excellent ["Crafting Interpreters" by Bob Nystrom](https://craftinginterpreters.com/). The simple/naive implementation was to practice a few things:

1. Working safely with Trait objects and downcasting.
2. Implementing a visitor pattern for expressions, objects, and statements.

This is only to satisfy my own interest in scanning and parsing, mostly – so work on this will be most likely incomplete, and changes will be few-and-far between.

### Running

Currently, this only runs by pointing the interpreter to a file. You'll need [`cargo`](https://www.rust-lang.org/learn/get-started) installed on your system (rustup will install it for you).

**Example:**

`cd` into `./interpreted`, and run:

```shell
cargo run -- ./test/function.lox
```

This will run the [`./test/function.lox`](test/function.lox) file.
