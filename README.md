# Actstore
A simple key/value store with useful functionality, potentially.
It's a hobby project while I learn Rust.
It lets you store arbitrary values to a store and then do things with it. Or just store it for reference.

## How
```sh
actstore set editrc kwrite ~/.bashrc
actstore get editrc
actstore exec editrc # runs as system command
actstore set mysite https://some.website.com
actstore url mysite # opens in web browser

```
