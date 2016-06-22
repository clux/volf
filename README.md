# volf
[![build status](https://secure.travis-ci.org/clux/volf.svg)](http://travis-ci.org/clux/volf)
[![coverage status](http://img.shields.io/coveralls/clux/volf.svg)](https://coveralls.io/r/clux/volf)
[![crates status](https://img.shields.io/crates/v/volf.svg)](https://crates.io/crates/volf)

A github webhook server in the style of [homu](https://github.com/barosl/homu).

## Usage
Add [volf](https://crates.io/crates/volf) to `Cargo.toml`.

## [documentation](http://clux.github.io/volf)

## Developing
To hack on `volf`, make debug builds and convenience link `volf` via `ln -sf $PWD/target/debug/volf /usr/local/bin/volf`.

When making changes:

```sh
cargo build
volf
cargo test # write tests
```

Before committing:

```sh
cargo fmt
```

## License
MIT-Licensed. See LICENSE file for details.
