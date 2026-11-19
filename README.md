![bark: Ark on bitcoin](assets/bark-ffi-header-white.jpg)

<div align="center">
<h1>bark-ffi: FFI bindings for Bark</h1>
<p>Built with uniffi-rs to generate native bindings for multiple languages from a single Rust codebase.</p>
</div>

<p align="center">
  <br />
  <a href="https://docs.second.tech">Docs</a> ·
  <a href="https://community.second.tech">Community</a> ·
  <a href="https://second.tech">Website</a> ·
  <a href="https://blog.second.tech">Blog</a> ·
  <a href="https://www.youtube.com/@2ndbtc">YouTube</a>
</p>

<br />

This repo is a language-agnostic Rust FFI implementation with support for different versions of uniffi.
For the actual bindings, go to [bark-ffi-bindings](https://gitlab.com/ark-bitcoin/bark-ffi-bindings).

## Structure

This repo can maintain different branches with different versions of uniffi, for example:

```
v0.32.0+bark-0.9.0.uniffi-0.31.2
v0.32.0+bark-0.9.0.uniffi-0.30.0
```

Bindings from [bark-ffi-bindings](https://gitlab.com/ark-bitcoin/bark-ffi-bindings) can tap into these branches to generate their own bindings:

```toml
# react-native/rust/Cargo.toml

[dependencies]
bark-ffi = { git = "https://gitlab.com/ark-bitcoin/bark-ffi", tag = "v0.32.0+bark-0.9.0.uniffi-0.31.2" }
uniffi = { version = "=0.31.2", features = ["cli"] }
```

## Build flavors

Two mutually-exclusive feature flags select the binding layer:

| Feature             | Target                       | Persister              |
| ------------------- | ---------------------------- | ---------------------- |
| `uniffi`            | native (Dart/Swift/Kotlin/…) | SQLite                 |
| `wasm-web`          | `wasm32-unknown-unknown`     | IndexedDB              |

`uniffi` is the default.

### Native (uniffi)

```sh
cargo build --no-default-features --features uniffi
```

### WASM

`secp256k1-sys` compiles a `wasm32` C object. Apple's stock clang has no
wasm32 backend; on macOS install LLVM clang first:

```sh
brew install llvm
rustup target add wasm32-unknown-unknown
```

On Linux the distro `clang` usually has wasm32 support built-in
(`apt install clang llvm`).

Set the env vars before building:

```sh
export CC_wasm32_unknown_unknown=/opt/homebrew/opt/llvm/bin/clang  # macOS
export AR_wasm32_unknown_unknown=/opt/homebrew/opt/llvm/bin/llvm-ar
cargo check --no-default-features --features wasm-web --target wasm32-unknown-unknown
```

### Nostr Wallet Connect

The `nwc` feature adds [NIP-47](https://github.com/nostr-protocol/nips/blob/master/47.md)
on top of either flavor, so an app can drive the wallet through a nostr relay:

```sh
cargo build --no-default-features --features uniffi,nwc
```

It is off by default: it pulls in the nostr stack and, natively, a TLS stack
for the relays. With it, `Wallet` gains `nwcAddRelay`, `nwcRemoveRelay`,
`nwcRelays`, `nwcCreateConnection`, `nwcConnections` and `nwcRevokeConnection`.
Add a relay, then create a connection and hand its URI to the app.

- Requests are answered while the daemon runs, so on mobile only while the
  app is. A connection created with the daemon stopped is served once it
  starts.
- The URI carries the app's secret key and cannot be shown again. Anyone
  holding it can use the connection until it is revoked.
- There are no spending limits: `pay_invoice` lets the app spend the whole
  balance.
- A URI names the relays it was created with. Removing them cuts its app off.
- Connections, with their service keys, are kept in `nwc.json` in the datadir
  (owner-only on unix), or in the IndexedDB database `<wallet database>.nwc`
  on wasm. Delete it along with the wallet's.
- Keep one open copy of a wallet, such as one browser tab: two copies each
  serve the connections they loaded and overwrite each other's changes.
