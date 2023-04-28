---
Title: Building a blog with Rust in 2023
Author: Louis
Date: 2023-04-23
Blurb: (the stupid way)
Tags: Rust, Web, Blog, HTML, template
---
# Building a blog with rust in 2023

As per the name, this blog is *imperfect*. I built it with the idea of making it
as easy as possible for me to get something released without *bikeshedding* too
much. Building it in rust made it easier to do that, but it still came at a
cost.

## Requirements

There were a few thing I was not willing to compromise on:

* I don't want to have to write HTML to write blog posts, markdown all the way
* I don't want to have to deal with NGINX (too many knobs to tune)
* I don't want to rely on any JavaScript (I'm already making you read bad
  prose, I can't also have you run awful JS)
* No PHP (it's personal I just don't like to write PHP)
* HTTP and HTTPS

## Nice to have

* Only one binary with minimal configuration
* Adding an article should not require me to restart the server
* It should run on Linode's base tier machine AKA: a toaster
* I'd like to be able to mix code and templates (PHP style)
* No need for docker
* It should be readable in [Links](http://links.twibright.com/)

## The Options I considered

### Any static site generator build around Markdown

Let's be honest. This blog is mainly composed of static pages. I could have
gotten away with using a static website generator and hosting the
HTML files on GitHub or something. But I wanted to avoid having individual HTML
pages for each page of </articles>. I looked into a couple of options, but
honestly, it looked more fun to build my own thing than to use someone else's.

### Briefly considering Python

I looked into using Python, more specifically
[Flask](https://flask.palletsprojects.com/en/2.2.x/), as I have used it in the
past, but having maintained Python projects, they tend to *rot*. Python lacking
a way to pin down a version of a dependency, and Pip being generally
unhygienic; I found that trying to get a Python project deployed outside a
Docker container is more complicated than it should be. And I did not want to
have to use Docker. I know there are ways to make it work, but I did not want
to end up in dependency hell, and there was something else I wanted to try...

### What about ~re~writing it in Rust?

I've been working in Rust for the last year or so at my `$JOB`, and I must
say, it has grown on me. Some parts of the languages are not as mature as I'd
like them to be (custom allocators, async traits, etc), but overall I'd say
it's a good replacement for projects where you'd typically reach for C++ or
Java. I've heard a few people talking about using it to build server backends,
and I wanted to learn more about the state of Rust frameworks for the web.

## The popular Rust web frameworks

Early on, I learned about different web frameworks, mostly through [Flosse's
rust web framework comparison
](https://github.com/flosse/rust-web-framework-comparison) rust web framework
comparison. The list makes it easy to know if a given framework support a
common feature, and I encourage anyone who thinks about using Rust to build web
applications to give it a look. It contains a list of both frontend frameworks
and backend frameworks. The frontend frameworks compile to WebAssembly; even
though it's not JavaScript, I still wanted to stay clear from requiring the
user to run code to display this website.

### Rocket

Even though [Rocket](https://rocket.rs/) is marked as "outdated" in [Flosse's
list](https://github.com/flosse/rust-web-framework-comparison), development
seems to still be going strong. In fact, the most recent commits are only about
2 weeks old, at the time of writing. Rocket is very much a "batteries included"
type of framework. I was quickly able to get an early version of the
</articles> page going, but I ended up not using it because I found the number
of dependencies a bit too high and the build times (on my old decaying laptop)
too slow for my liking. It looks to be a great framework that comes with
everything you would need to build complex websites with forms and stuff, but
it felt overkill for my usage.

### Warp

I was looking for a framework that would be a tad smaller Having used
[warp](https://docs.rs/warp/latest/warp/) as my `$JOB` before, I briefly
considered it. Warp is built on top of Rust Generics and its type system. This
means that a lot of it feels magic, just add a few filters and some
`serde::Desericalise` implementing types, and you'll have a working API
endpoint in no time... Except that warp, due to its *liberal* use of generics,
it contributes a lot to the overall time it takes to build our projects. But my
biggest gripe with warp is that when things go wrong (which is a compile-time,
at least) it generates compile errors that compete with some of the worse C++
template errors I've had the displeasure of seeing.

### Actix Web

[Actix Web](https://actix.rs/docs/whatis) describes itself as a
"micro-framework" (much like flask is often described). It handles routing,
HTTP/1, HTTP/2, HTTPS and typed HTML queries (`q?key=value`). The only thing I
needed was templating. It required an `async main` since it's built on top of
tokio, but it does not manage every part of the program the same way Rocket
would. To me, it felt easier to compose with other libraries, so I stuck with
it.

## HTML templating

When I was experimenting with Rocket, I also tried
[handlebars](https://docs.rs/handlebars/latest/handlebars/) as the main
templating engine. It worked well, but to me, it felt awkward to have the code
and the format in 2 different places. The last time I did any kind of web
development was back in college, most of which was done with PHP. Although I
don't really like PHP, there was one thing I really liked (and apparently other
people don't): you can mix HTML and code.

### typed-html

When I found [typed-html](https://github.com/bodil/typed-html) it seemed to be
exactly what I was looking for, I could embed HTML through the `html!` macro
and I could use rust expressions within that macro to build web pages
server-side. The first page I build was the </articles> page and I quickly ran
into a limitation of typed-html, due to typed-html's goal of making it easy to
build correct HTML through type safety it won't allow you to use a code block
as the first child of certain tags.

```html
<!-- Not allowed -->
<head> { /* rust code */ } </head>

<!-- Ok  -->
<head>
    <h1>"A title"</h1>
    { /* rust code */ }
</head>
```

This is done so that It can guarantee a certain level of correctness, IE: no
`<head>`s in `<head>`s. I wanted to have functions to define common headers and
common footers for each page, but this limitation made it pretty awkward. One
thing I took away from the experiment (probably the wrong one based on the
library's name): I could use Rust macros to embed arbitrary tokens in my Rust
code.

## Building my own version of the `html!` macro

I wanted to be able to reference variables and evaluate expressions, not just
enumerations, in `{ }` brackets. I found typed-html pretty limiting, and I
hit maximum recursion a few times while trying to build fairly simple pages; I
had to write my own. I won't go into details, but I made the code available on
GitHub under <https://github.com/lorlouis/html_template>, the code is
definitely not perfect, but it worked well enough to build this blog.

With that, I had all the elements I needed to build this blog.

Here's part of the code I use to turn a Markdown file into an article

```rust
...
let body: Root = html!{
    <!DOCTYPE html>
    <html>
        <head>
            {common_head(real_title.clone(), author.cloned(), blurb.cloned())}
        </head>
        <body>
            <header>
            { common_header() }
            </header>
            <main>
            { markdown.to_html() }
            </main>
            <footer>
            { common_footer() }
            </footer>
        </body>
    </html>
}.into();
...
```

## Final Thoughts

In the end, I built a fairly unsophisticated blog using mostly pre-existing
libraries. The downside to this approach is that while I was paying attention
to not pulling in too many dependencies, I now depend on 168 external
dependencies. Using Actic-web as it made routing and handling query parameters
really easy. I'm also glad I build
[html\_template](https://github.com/lorlouis/html_template) as it was the first
time I had ever used Rust's proc-macros, and it made building HTML pages
in-code much easier.
