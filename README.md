# ipclc

`ipclc` is a cross-platform Rust command-line network calculator for IPv4 and IPv6. It accepts CIDR notation and dotted IPv4 masks, and produces human-readable or JSON output.

## Build and test

Requirements: current stable Rust and [just](https://github.com/casey/just). CI also checks the minimum supported Rust version (1.84.0).

## Homebrew

Once the `psvmcc/homebrew-tap` tap repository contains `Formula/ipclc.rb`, install with:

```console
brew tap psvmcc/tap
brew install psvmcc/tap/ipclc
```

The release formula installs a prebuilt platform binary from GitHub Release and verifies its SHA-256. It does not require Rust or Cargo on the user's machine. Release tags update `psvmcc/homebrew-tap` through the `HOMEBREW_TAP_TOKEN` repository secret.

```console
just check
just build
```

If `just` is unavailable, run the equivalent Cargo commands:

```console
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```

## Usage

```console
ipclc 192.168.10.42/24
ipclc 192.168.10.42 255.255.255.0
ipclc 2001:db8:1234::42/64 --json
ipclc 10.0.0.0/8 --contains 10.20.30.40
ipclc 192.168.1.0/24 --split 26
ipclc 2001:db8::42/128 --reverse-dns
```

Useful options:

- `--json` produces stable machine-readable JSON.
- `--short` prints only the normalized network CIDR.
- `--contains <IP>` checks network membership.
- `--split <PREFIX>` lists child networks (up to 4096 results per invocation).
- `--reverse-dns` prints the full reverse-DNS name for the supplied address.
- `--reverse-zone` prints the reverse-DNS zone for an aligned prefix.
- `--overlaps <NETWORK>` checks overlap with another network.
- `--aggregate <NETWORKS>` aggregates comma-separated adjacent prefixes.
- `--neighbor previous|next` prints the adjacent prefix.
- `--completions bash|zsh|fish|powershell|elvish` generates shell completions.
- `--color auto|always|never` controls ANSI colors.

IPv4 output includes network, broadcast, wildcard, host range, and usable host count. IPv6 output includes the normalized prefix, first/last addresses, and total address count. IPv6 output does not contain a broadcast field because IPv6 has no broadcast mechanism.

## License

The project is distributed under the MIT License. Dependencies retain their upstream permissive licenses; the approved license policy is documented in `deny.toml`.
