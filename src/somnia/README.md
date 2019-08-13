# Somnia Lisp

*Somnia Lisp* is a Lisp for numeric calculation.
It aimed to be embed to generate sound signal.

*Tapir Lisp* eats *Somnia Lisp* code then yield a sound module.
It's not a conpound sound module, but a programmable sound module.

S-expression reader and printer are placed at [../sexp.rs](../sexp.rs).

*Somnia Lisp* will have some features below:

- unsigned integer type
    - they are finally automatically converted Rust's `f64`
- **overflowing**
- four basic operators of arithmatic
- bit sifting
- exponential
- square root
