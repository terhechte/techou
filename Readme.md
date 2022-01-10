<img src="Design/logo.svg" />

# techou [てちょう]

## Notebook, Memo pad, (pocket) diary

### A terrible static site generator

Techou was the second Rust project I started, after the horribly failed [Wanderduene](https://github.com/terhechte/wanderduene/)
static site generator in early 2018.

Techou is a static site generator too, and you really shouldn't use it. The code is terrible, there's an impressive lack of tests,
the structure is weird, and the features are very much made for my requirements. Also, did I mention that there's no documentation?

I still use it for my own projects because the simplicity of the code allows me to quickly hack in features I need.

I used to run techou for [Appventure.me](https://appventure.me) where it builds not just posts but also collections of posts (called books)
such as [this book on Swift Keypaths](https://appventure.me/guides/keypaths/intro.html).

### Features

- Built in Javascript search via elasticlunr
- Support for posts, pages, custom content and *books*.
- Link resolver, so that `[lnk:my-page]` is resolved to the correct url (e.e. `/books/long-book/chapter3/my-page.html`).
- Code syntax highlighting via syntect (or via Splash for Swift code)
- A build cache to allow faster builds for complex sites
- A build-in webserver with Websocket support. Techou scans any changes to your files (e.g. templates or posts) and if there's a change, it will tell the currently open browser to reload the current pages. This is great for editing
- Support for custom metadata in the config
- A commandline tool to create new projects or new posts (`techou new`)
- Automatic generation of sidebars with chapter headers for easy navigation in sites.
- RSS Feed generation
- All this is pretty undocumented. The best way to understand how to use it is to read the [Configuration](src/config.rs) and to have a look at the [test site](site/) or the codebase for [appventure.me](https://github.com/terhechte/appventure)
- use `normalized_damerau_levenshtein` to find similar content for posts.
- It uses `Tera` as a templating language. I was too lazy to document all the template tags and variables. Look at the `site` or `appventure` again.

### State

This is/was my second proper Rust project. 

I cobbled the initial version of `techou` together in a weekend and then spend a couple of weeks improving it on the side. It aims to be a very hackable static site generator with obvious and simple defaults.

I'll continue hacking on this for my personal needs. The quality of everything is terrible though. I have no plans on cleaning it up.

### Usage

``` sh
techou -h
```

