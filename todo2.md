# Changes required for 1.0
- allow starting with --guide or --post and then only this post / guide will be re-rendered, the rest will be ignr0ed
- instead of `format!` use proper `PathBuf` code everywhere to generate folders, slugs (or, define our own type for this)
- use multi_try crate for error handling
- expose enough functions to re-implement executor in bin
- all documents (from guides, posts, etc) should be registered in one state,
  with their type, to easily iterate over them, say for tags Vec<(Document, DocumentType)>
- should crash if baseURl is not set
- the webserver/websocket should tell the browser the one page to reload and hand in the html over the websocket if only one page was reloaded
- make the webserver an optional dependency
- all the html parser flags should be disableable in the config
- move all the html-in-techou (i.e. <h1>section</h1>) into config strings. the config string can also be "filename:templates/section.html" for template files instead
- cleanup the messy unwrap and so on that I have and instead buy into nice error handling
- display errors in the browser at the top if the websocket is on

- bindings for various languages

- add support for my mdbook extensions for various markdown parsings
- use techou as a library for this (temporary, a local)
- improve the parsing speed for MD (see appventure)
- iterate over the MD only once, have all the code that modifies the MD in that one iteration
- use salsa to figure out the dependency tree (via the template parser) and only rerender the dependencies that changed
