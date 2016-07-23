# volf
[![build status](https://secure.travis-ci.org/clux/volf.svg)](http://travis-ci.org/clux/volf)
[![coverage status](http://img.shields.io/coveralls/clux/volf.svg)](https://coveralls.io/r/clux/volf)
[![crates status](https://img.shields.io/crates/v/volf.svg)](https://crates.io/crates/volf)

A github webhook server and pull request approval bot in the style of [homu](https://github.com/barosl/homu).

## Usage

1. Create a github machine account for volf.  Register a new application in its [account settings](https://github.com/settings/applications), and set OAuth Callback URL to `http://HOST:54857/callback`. Generate an personal access token for the machine account with scopes: ` public_repo, read:repo_hook, repo:status`.

2. Add a webhook to your repositories:

 - Payload URL: `http://HOST:54857/github`
 - Content type: `application/json`
 - Secret: A repo-wide unique secret for `volf.json` (under `github.secret`)
 - Events: *Issue comment* + *Pull request* + *Push*

3. Install and configure run this application somewhere with you own [volf.json](./volf.json).

```sh
cargo install volf
export RUST_LOG=info
volf config generate
volf config edit
export GITHUB_TOKEN=personal_access_token_from_above
volf
```

4. Wait for @clux to implement stuff.

## Developing
To hack on `volf`, make debug builds and convenience link `volf` via `ln -sf $PWD/target/debug/volf /usr/local/bin/volf`.

When making changes:

```sh
cargo build
RUST_LOG=hyper=info,volf=debug volf
cargo test
cargo fmt
```

## License
MIT-Licensed. See LICENSE file for details.
