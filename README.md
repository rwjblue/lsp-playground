# lsp-playground

Repo for playing around with writing a custom LSP in Rust. The overall goal is
to make the from "zero" to something useful pretty easy.

I'd like to:

- better understand the LSP protocol
- implement diagnostics for custom functionality
- implement a basic code action

## References

- [Language Server Protocol
  Spec](https://microsoft.github.io/language-server-protocol/specifications/specification-current)
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) - A crate that wraps
  [Tower](https://github.com/tower-rs/tower) and provides a nice language server
  trait that we can implement to get off the ground floor very easily.
- [lsp-types](https://github.com/gluon-lang/lsp-types) - A crate that provides
  _basically_ all of the types needed to work with LSPs.

## Learnings

You **must** opt-in to server capabilities that you need in order to do
anything useful in the language server. By default, it seems that the only
hooks that are called without explicitly opting in to are `initialize` (which
is where you should actually do the "opting in") and `shutdown`.
