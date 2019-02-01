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
- [ ] commandline tool to create new posts
- [ ] have 'release' flag to build with 'published' only, no websockets, and maybe even parcel or webpack?
- [ ] consider 'https://github.com/nathan/pax' to compress JS in release and the css-.. crate for compressing CSS
- [ ] tags
- [ ] archives
- [ ] pagination
- [ ] next / previous article
- [ ] article recommendations
- [ ] cleanup bin/techou.rs
- [ ] cleanup server
- [ ] use salsa to only re-render what has changed
- [ ] some restructuring to make it easier usable as a library
