# Tapir Lisp

*Tapir Lisp* is a Lisp-like language for describing a directed graph (it's not a tree); it define a sound signal flow.
Note that *Tapir Lisp* does not have conditional expressions, functions and thus local bindings.
It has some features below:

- create unit generator instances
- define sound environment (e.g. bpm)
- define global variables
    - for creating multiple signal output

S-expression reader and printer are placed at [../sexp.rs](../sexp.rs).

## Unit generators

Unit generators generate sound signal.
The output signals may be depend on values from other unit generator's or not.

For details, see `make_unit()` in `./eval.rs`.

## Special forms

There are some types of special forms.
`def` and environment modifications are that.

`def` evaluates the secound argument and set it to the name of first argument in global binding.
If set to same name twice *Tapir Lisp* will cause error.

Environment modifications are simply modifying bpm or measure via `bpm` or `measure`.
