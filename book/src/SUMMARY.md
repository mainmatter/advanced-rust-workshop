# Summary

[Welcome](00_intro/00_welcome.md)

- [Names and docs](01_api_design/00_intro.md)
  - [Naming conventions](01_api_design/01_naming.md)
  - [Doc comments](01_api_design/02_doc_comments.md)

- [The newtype pattern](02_newtype/00_intro.md)
  - [Semantic confusion](02_newtype/01_semantic_confusion.md)
  - [Parse, don't validate](02_newtype/02_parse_dont_validate.md)
  - [Is it encapsulated?](02_newtype/03_encapsulation.md)

- [Common traits](03_common_traits/00_intro.md)
  - [Debug and Display](03_common_traits/01_debug.md)
  - [Eq, Hash and conversions](03_common_traits/02_hash_eq.md)

- [Ownership, borrowing and lifetimes](04_borrowing/00_intro.md)
  - [Aliasing XOR mutability](04_borrowing/01_retain.md)
  - [Ownership in signatures](04_borrowing/02_ownership.md)

- [RAII](05_raii/00_intro.md)
  - [Drop guards](05_raii/02_drop_guard.md)
  - [Drop bombs and the limits of Drop](05_raii/03_drop_bomb.md)
  - [Closure APIs](05_raii/04_closure_api.md)

- [Typestate](06_typestate/00_intro.md)
  - [States as capabilities](06_typestate/01_transaction.md)
  - [States that move](06_typestate/02_writer.md)

- [Extension traits](07_extension_traits/00_intro.md)
  - [Which type to implement, and when not to](07_extension_traits/01_str_ext.md)
  - [Extending a trait](07_extension_traits/02_iter_ext.md)

- [Polymorphism](08_polymorphism/00_intro.md)
  - [Generics and `dyn`](08_polymorphism/01_format.md)
  - [Sealed traits](08_polymorphism/02_sealed.md)

- [PhantomData, variance and brands](09_phantom/00_intro.md)
  - [Lifetimes for things that are not references](09_phantom/01_entry_ref.md)
  - [Branded lifetimes](09_phantom/02_branded.md)
