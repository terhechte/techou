# Changes required for 1.0
- allow starting with --guide or --post and then only this post / guide will be re-rendered, the rest will be ignr0ed
- instead of `format!` use proper `PathBuf` code everywhere to generate folders, slugs
- use multi_try crate for error handling
- expose enough functions to re-implement executor in bin
- all documents (from guides, posts, etc) should be registered in one state,
  with their type, to easily iterate over them, say for tags Vec<(Document, DocumentType)>

- bindings for various languages

- support for multiple "books"
- books contain (like mdbook) a folder structure and a main .md file that shows the structure (like mdbook)
- add support for my mdbook extensions for various markdown parsings
- use techou as a library for this (temporary, a local)
- improve the parsing speed for MD (see appventure)
- the 'apv' prefix for code has to go into the config
- iterate over the MD only once, have all the code that modifies the MD in that one iteration
