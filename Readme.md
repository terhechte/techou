<img src="assets/logo.svg" />

# techou [てちょう]

## Notebook, Memo pad, (pocket) diary

### A terrible static site generator


I cobbled the initial version of `techou` together in a weekend and then spend a couple of weeks improving it on the side. It aims to be a very hackable static site generator with obvious and simple defaults.

The idea is that a good collection of primitives should enable eager devs to compose a static site just to their liking. Currently, this is not really the case as I'm working on improving the abstractions.

However, what it is right now is a very nice and kinda configurable static site generator.

## Here's what it can do
- By default, just generate a `html` folder from the project in the current folder (see below for default project setup)
- If you need flexibility, run `techou create` which will create a `project.toml` in the current directory. This can be used to configure `techou`. (For more info, look into `config.rs`
- A buildin development server (run techou via `techou -s`)
- Auto-rebuild the site if a file changes (run techou via `techou -w`)
- Auto-reload of the site in a browser (if it is currently open) when anything changes (via websockets)
- A couple of useful Markdown extensions
- A simple commandline tool `techou` with the ability to create a new post with the proper front matter `techou new`
- Categorizes posts by year/month, tag, and keyword
- Documents are written in markdown and contain a "front matter" that allows you to define settings
- Support for additional documents that are not posts (pages)
- Support for additional meta information in the document front-matter that canb e accessed from the template
- A terrible sample project (see `site`)

So, if you have a project in `project.toml` running it with auto-reload and server enabled, do:

``` bash
techou -f project.toml -ws
```

More information will come. The project is still pretty much in development.



## Open Tasks

- [ ] if therr're rendering errors, end them via websocket to the browser, so that they can be displayed in a small dark bottom-bar in the rendered site...
- [ ] if the terminal supports colors, use that
- [ ] consider 'https://github.com/nathan/pax' to compress JS in release and the css-.. crate for compressing CSS
- [ ] figure out error reporting now that some funcs went away from the executor, but we still want to continue building even if one page fails...
- [ ] next / previous article
- [ ] have a function to get a link to a different article (maybe by article id?, might need an article id)
- [x] article recommendations
# Long Term Future
- [ ] use salsa to only re-render what has changed
- [ ] some restructuring to make it easier usable as a library
