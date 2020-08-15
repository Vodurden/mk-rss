# Overview

An aws lambda to generate a RSS page from websites that no longer provide a RSS feed. Written in rust!

# Releasing

1. Enter `nix-shell` (optional if `direnv` is installed)
2. Build `mk-rss` statically: `cargo build --target x86_64-unknown-linux-musl`
3. Authenticate with AWS
4. Execute `./auto/deploy`
