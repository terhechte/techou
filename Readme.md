てちょう
techou
手帳

notebook, memo pad, (pocket) diary 

[WIP]
A rust static site engine written in a day. 
Extended over a week
Designed to be easy to hack on

[Open Tasks]
- [x] Use `err-derive` crate: https://crates.io/crates/err-derive
- [x] Use `err-ctx` crate: https://crates.io/crates/err-ctx
- [ ] if therr're rendering errors, end them via websocket to the browser, so that they can be displayed in a small dark bottom-bar in the rendered site...
- [ ] if the terminal supports colors, use that
- [x] commandline tool to create new posts
- [x] have 'release' flag to build with 'published' only, no websockets, and maybe even parcel or webpack?
- [ ] consider 'https://github.com/nathan/pax' to compress JS in release and the css-.. crate for compressing CSS
- [ ] when there is a project.toml, it should be used in stead of just using "."
- [ ] tags
- [ ] archives
- [ ] support the better category / sitecategory system from rusttest1 instead of my year/month. that way I can also do tags etc
- [ ] pagination
- [ ] add config to all templates, also allow meta information in config just like with articles.
- [ ] next / previous article
- [ ] have a function to get a link to a different article (maybe by article id?, might need an article id)
- [ ] article recommendations
- [ ] cleanup bin/techou.rs
- [ ] cleanup server
- [ ] use salsa to only re-render what has changed
- [ ] some restructuring to make it easier usable as a library
in this r code:
``` R
x <- 5
y <- 6
x + y
```
the last y is missing. this seems to be a syntect issue. as the following also happens
``` R
x + y
y + z
x + a
```
y, z, a on the right side never appear either
