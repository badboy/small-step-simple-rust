# small-step approach to the SIMPLE language in Rust

[Blog post about my first experience with Rust](https://fnordig.de/2014/08/12/first-experience-with-rust/)

---

This is an implementation of the small-step approach to the SIMPLE language as introduced by
[Tom Stuart](https://twitter.com/tomstuart) in "Understanding Computation", Chapter 1, "The Meaning of Programs".
See his website: <http://computationbook.com/>.

The usage is pretty simple. As there is no parser for SIMPLE (yet?) you have to write the AST
yourself. A few macros are provided for easy access. You can then create a virtual machine and
pass this AST plus an environment hash. When calling `run`, the machine steps through the code,
reducing it until it reaches a point where no further reduction is possible.

```rust
let mut env = HashMap::new();
env.insert("y".to_string(), number!(1));

let mut m = Machine::new(
    sequence!(
        assign!("x", number!(3)),
        assign!("res", add!(add!(number!(38), variable!("x")), variable!("y")))
        ),
        env
    );

m.run();
// At this point `res` in the HashMap will be `Number(42)`
```


The code is much larger as the equivalent Ruby code. This is both due to the restricitions
of Rust (explicit types and everything, a good thing) and my non-existing experience with Rust
at all (this is my first Rust code larger than a simple "Hello World")


## License

Just like the [original Ruby source code](https://github.com/tomstuart/computationbook), this Rust source code is released under the [CC0 1.0 Public Domain Dedication](http://creativecommons.org/publicdomain/zero/1.0/), which means that you can do whatever you like with it.
