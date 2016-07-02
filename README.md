# volf
[![build status](https://secure.travis-ci.org/clux/volf.svg)](http://travis-ci.org/clux/volf)
[![coverage status](http://img.shields.io/coveralls/clux/volf.svg)](https://coveralls.io/r/clux/volf)
[![crates status](https://img.shields.io/crates/v/volf.svg)](https://crates.io/crates/volf)

A github webhook server and pull request approval bot in the style of [homu](https://github.com/barosl/homu).

## Usage

1. Install and run this application somewhere with you own [volf.json](./volf.json)

2. Create a github machine account for volf.  Register a new application in his [account settings](https://github.com/settings/applications), and set OAuth Callback URL to `http://HOST:54857/callback`

3. Add a webhook to your repository:

 - Payload URL: `http://HOST:54857/github`
 - Content type: `application/json`
 - Secret: Corresponding repository's `github.secret` in `volf.json`
 - Events: *Issue comment* +  *Pull request* + *Push*

4. Wait for @clux to implement stuff. Currently this is just a webhook server.

## Developing
To hack on `volf`, make debug builds and convenience link `volf` via `ln -sf $PWD/target/debug/volf /usr/local/bin/volf`.

When making changes:

```sh
cargo build
RUST_LOG=hyper=info,volf=debug volf
cargo test # write tests
cargo fmt
```

## License
MIT-Licensed. See LICENSE file for details.
