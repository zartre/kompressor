# Kompressor

Kompressor is a cross-platform image compression application for
macOS, Linux and Windows built using Rust.

## Building

Place your certificate (`.p12`) file at `secrets/kompressor.p12`.

Put the password of the certificate into the `APPLE_SIGNING_PASSWORD`
environment variable. It is used by `cargo-bundle`.

Build and run with the standard Cargo workflow:

```sh
cargo build --release
cargo bundle --release --binary-path target/release/kompressor
```
