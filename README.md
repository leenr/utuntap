# Universal Tun/Tap for Rust

[![Crates.io](https://img.shields.io/crates/v/utuntap)](https://crates.io/crates/utuntap)

This library aims to provide robust, well-tested, unified APIs to open Tun/Tap devices on different OSes. It is runtime-agnostic so that you can create your cross-platform wrappers for [Tokio](https://tokio.rs), [async-std](https://github.com/async-rs/async-std) and so on.

## Usage

It simply provides options to open Tun/Tap device files. More options are listed in the [documentation](https://docs.rs/utuntap).

```rust
use utuntap::tun;


let (mut file, filename) = tun::OpenOptions::new()
    .packet_info(false) // Only available on Linux
    .open(10)
    .expect("failed to open device");
```

## Support Platforms

| OS      | CI&nbsp;Status                                                                                                                        | Comment                                                                                                                                                                                                                                                                              |
|---------|---------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Linux   | [![Build Status](https://travis-ci.org/siegfried/utuntap.svg?branch=master)](https://travis-ci.org/siegfried/utuntap)                 | `musl` is also supported.                                                                                                                                                                                                                                                            |
| OpenBSD | [![builds.sr.ht status](https://builds.sr.ht/~siegfried/utuntap/.build.yml.svg)](https://builds.sr.ht/~siegfried/utuntap/.build.yml?) | According to the [manual](https://man.openbsd.org/tun.4), each packet read or written is prefixed with a tunnel header consisting of a 4-byte network byte order integer containing the address family. The values are listed [here](https://man.openbsd.org/netintro.4#ADDRESSING). |
| FreeBSD | Incomplete. Receive integration test is failing.                                                                                      | Mostly uses the same code path as for OpenBSD.                                                                                                                                                                                                                                       |
| macOS   | TUN-only                                                                                                                              | A 4-byte address family prefixed is required by read and write: `[0u8, 0, 0, 2]` for IPv4; `[0u8, 0, 0, 10]` for IPv6.                                                                                                                                                               |
