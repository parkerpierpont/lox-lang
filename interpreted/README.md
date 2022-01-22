NOTE:

I started this in rust, and got bored when trying to write an interpreter that's going through all sorts of different scopes – and found it a bit of a hassle to do some of the backtracking (re-writing) required to follow the book – since it's much more cumbersome to re-architect something in rust from a book, and then have to re-implement it again in rust after it's changed later on.

Since I am implementing this in rust anyway, as a learning excercise, I decided to move onto the Bytecode VM, after functions were implemented in the interpreted version – since that portion of the book is written in C, and will cover some of the issues that I was basically trying to figure out myself (poorly).

Also, I didn't feel like trying to implement basically a terrible garbage collector in rust that somehow matched the semantics of the book. So, I'm dipping in the bytecode VM portion of the book, after completing basically everything except for classes. See the `/vm` folder in the root for that portion.
