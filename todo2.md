# Changes required for 1.0
- use multi_try crate for error handling
- expose enough functions to re-implement executor in bin
- all documents (from guides, posts, etc) should be registered in one state,
  with their type, to easily iterate over them, say for tags Vec<(Document, DocumentType)>

- support for multiple "books"
- books contain (like mdbook) a folder structure and a main .md file that shows the structure (like mdbook)
- add support for my mdbook extensions for various markdown parsings
- use techou as a library for this (temporary, a local)
- improve the parsing speed for MD (see appventure)
