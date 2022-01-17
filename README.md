## Lox Implementation

A super-naive implementation of Lox from the excellent ["Crafting Interpreters" by Bob Nystrom](https://craftinginterpreters.com/). The simple/naive implementation was to practice a few things:

1. Working safely with Trait objects and downcasting.
2. Implementing a visitor pattern for expressions, objects, and statements.

### Running

Currently, this only runs by pointing the interpreter to a file. You'll need [`cargo`](https://www.rust-lang.org/learn/get-started) installed on your system (rustup will install it for you).

**Example:**

`cd` into the root of the repo, and run:

```shell
cargo run -- ./test/function.lox
```

This will run the [`./test/function.lox`](test/function.lox) file.
