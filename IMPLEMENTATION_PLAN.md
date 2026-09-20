# Implementation Plan for `ipclc`

## 1. Project Goal

Build a cross-platform command-line utility in Rust that works similarly to `ipcalc` and supports both IPv4 and IPv6 networks.

The utility must run on Linux, macOS, and Windows.

Supported input examples:

```console
ipclc 192.168.1.42/24
ipclc 192.168.1.42 255.255.255.0
ipclc 2001:db8:1234::42/64
```

## 2. Core Features

### IPv4

Calculate and display:

- IP address and address type
- CIDR prefix and network mask
- Wildcard mask
- Network and broadcast addresses
- First and last usable host
- Total address and usable host counts

Special handling is required for `/0`, `/31`, and `/32` networks.

### IPv6

Calculate and display:

- Compressed and expanded addresses
- Address type
- CIDR prefix and network mask
- Network address
- First and last address in the prefix
- Total address count

IPv6 does not have broadcast addresses. The broadcast field should be omitted or shown as not applicable. Special handling is required for `/0`, `/127`, and `/128` networks.

## 3. Command-Line Interface

Proposed syntax:

```text
ipclc [OPTIONS] <ADDRESS> [NETMASK]
```

Initial options:

```text
-j, --json          Print output as JSON
-s, --short         Print a shortened result
-c, --color <MODE>  Set color mode: auto, always, or never
-n, --no-color      Disable colored output
    --contains <IP> Check whether an address belongs to the network
    --split <PREFIX> Split the network into smaller subnets
-V, --version       Print version information
-h, --help          Print help
```

Examples:

```console
ipclc 192.168.1.42/24
ipclc 192.168.1.42 255.255.255.0
ipclc 10.0.0.0/8 --contains 10.20.30.40
ipclc 192.168.1.0/24 --split 26
ipclc 2001:db8::1/64 --json
ipclc 2001:db8::/48 --split 64
```

## 4. Project Structure

```text
ipclc/
├── Cargo.toml
├── README.md
├── LICENSE
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli.rs
│   ├── error.rs
│   ├── network/
│   │   ├── mod.rs
│   │   ├── ipv4.rs
│   │   └── ipv6.rs
│   └── output/
│       ├── mod.rs
│       ├── text.rs
│       └── json.rs
└── tests/
    ├── cli.rs
    ├── ipv4.rs
    ├── ipv6.rs
    └── snapshots/
```

Responsibilities:

- `main.rs`: application startup and exit codes
- `lib.rs`: public library interface
- `cli.rs`: command-line argument parsing
- `error.rs`: application error types
- `network/ipv4.rs`: IPv4 parsing and calculations
- `network/ipv6.rs`: IPv6 parsing and calculations
- `output/text.rs`: human-readable output
- `output/json.rs`: stable machine-readable output
- `tests/`: integration and end-to-end tests

Network calculation logic must remain independent from the CLI and output formatting.

## 5. Dependencies

Suggested dependencies:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anstream = "0.6"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
proptest = "1"
insta = "1"
```

Network calculations can either use the `ipnet` crate or be implemented directly with `u32` for IPv4 and `u128` for IPv6. Direct implementation provides more control over edge cases and output compatibility.

## 6. Implementation Stages

### Stage 1: Project Setup

- Create the Cargo project with library and binary targets.
- Configure formatting and linting.
- Add `clap` argument definitions.
- Define application exit codes.
- Configure CI for Linux, macOS, and Windows.

Suggested exit codes:

```text
0  Successful execution
1  Internal or unexpected error
2  Invalid arguments, address, mask, or prefix
```

### Stage 2: Input Parsing

Implement parsing for:

- IPv4 CIDR notation
- IPv4 address with a separate dotted-decimal mask
- IPv6 CIDR notation
- Compressed IPv6 addresses
- IPv4-mapped IPv6 addresses

Validate:

- IPv4 prefix range: `0..=32`
- IPv6 prefix range: `0..=128`
- IPv4 masks contain contiguous set bits
- Address and mask families are compatible
- Required arguments are present
- Conflicting options are rejected

Invalid examples:

```console
ipclc 192.168.1.1/33
ipclc 2001:db8::1/129
ipclc 192.168.1.1 255.0.255.0
ipclc 2001:db8::1 255.255.255.0
```

### Stage 3: IPv4 Calculations

Implement:

```text
network   = address AND netmask
wildcard  = NOT netmask
broadcast = network OR wildcard
```

Calculate the network mask, prefix, wildcard, network, broadcast, usable host range, total address count, and usable host count.

Handle `/0`, point-to-point `/31`, and single-host `/32` networks. Classify unspecified, private, shared, loopback, link-local, documentation, benchmarking, multicast, reserved, and global unicast addresses.

### Stage 4: IPv6 Calculations

Convert IPv6 addresses to `u128` for bitwise calculations. Calculate the prefix mask, network, first and last addresses, compressed and expanded forms, and address count.

Handle `/0`, point-to-point `/127`, and single-address `/128` prefixes. Classify unspecified, loopback, IPv4-mapped, unique-local, link-local, documentation, multicast, and global unicast addresses. Optionally determine multicast scope.

Because `2^128` cannot be stored in `u128`, use a dedicated representation:

```rust
enum AddressCount {
    Value(u128),
    FullIpv6Space,
}
```

### Stage 5: Common Result Model

Separate calculations from presentation:

```rust
pub enum NetworkInfo {
    V4(Ipv4NetworkInfo),
    V6(Ipv6NetworkInfo),
}
```

Use `u64` for IPv4 address counts because an IPv4 `/0` contains `2^32` addresses and does not fit in `u32`.

### Stage 6: Output Formatting

Implement human-readable, short, and JSON output modes.

Requirements:

- Keep calculations out of formatting code.
- Disable colors when stdout is not a terminal.
- Never include ANSI escape sequences in JSON.
- Keep JSON field names stable between releases.
- Serialize large counts as strings when necessary.
- Write normal output to stdout and errors to stderr.

### Stage 7: Additional Network Operations

After the core implementation is stable, add:

- Address membership checks
- Subnet splitting
- Adjacent subnet calculation
- Network overlap checks
- Prefix aggregation
- Reverse DNS generation using `in-addr.arpa` and `ip6.arpa`

## 7. Testing Plan

### Unit Tests

Write unit tests next to the implementation modules.

IPv4 tests must cover:

- Prefix-to-mask conversion for every prefix from `/0` through `/32`
- Mask-to-prefix conversion
- Rejection of non-contiguous masks
- Network, broadcast, and wildcard calculations
- Host range and address counts
- Address classification
- Edge cases including `/0`, `/31`, and `/32`

Required IPv4 cases:

```text
0.0.0.0/0
10.0.0.1/8
192.168.1.42/24
192.0.2.1/31
192.0.2.1/32
255.255.255.255/32
```

IPv6 tests must cover:

- Prefix mask generation for every prefix from `/0` through `/128`
- Network, first-address, and last-address calculations
- Expanded and compressed formatting
- Address counts
- Address classification
- Edge cases including `/0`, `/127`, and `/128`

Required IPv6 cases:

```text
::/0
::/128
::1/128
2001:db8::1/64
2001:db8::1/127
2001:db8::1/128
ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff/128
```

### Property-Based Tests

Use `proptest` to verify these invariants for any valid network:

```text
network <= address <= last_address
address AND mask == network
network has all host bits set to zero
last_address has all host bits set to one
```

Additional IPv4 invariants:

```text
broadcast == network OR wildcard
netmask AND wildcard == 0
```

Test prefix/mask and address/integer round trips.

### CLI Integration Tests

Use `assert_cmd` and `predicates` to launch the compiled binary and verify:

- Successful execution
- stdout and stderr content
- Exit codes
- `--help` and `--version`
- CIDR and separate IPv4 mask input
- JSON and short output
- Invalid addresses, masks, and prefixes
- Conflicting options
- IPv4 and IPv6 membership checks

Example:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn prints_ipv4_network_information() {
    let mut cmd = Command::cargo_bin("ipclc").unwrap();

    cmd.arg("192.168.10.42/24")
        .assert()
        .success()
        .stdout(predicate::str::contains("Network:"))
        .stdout(predicate::str::contains("192.168.10.0"))
        .stdout(predicate::str::contains("192.168.10.255"));
}
```

### JSON Contract Tests

Deserialize JSON output and verify:

- Output is valid JSON.
- Field names remain stable.
- IPv4-only fields are absent from IPv6 output.
- Large IPv6 counts are not truncated.
- Numeric and string representations match the documented contract.
- JSON never contains ANSI color codes.

Do not test JSON only as raw text because field ordering should not be part of the public contract.

### Snapshot Tests

Use `insta` for deterministic human-readable output snapshots with `--color never`.

Create snapshots for:

- A normal IPv4 network
- IPv4 `/31` and `/32`
- A normal IPv6 network
- IPv6 `/127` and `/128`
- Invalid input errors
- Help output

### Cross-Platform Tests

Run tests on Ubuntu, macOS, and Windows. Avoid assumptions about path separators, terminal capabilities, shell quoting, and line endings. Normalize line endings in snapshots if necessary.

### Regression Tests

Add a regression test for every confirmed bug before or together with its fix. Examples:

```rust
#[test]
fn ipv4_slash_zero_does_not_overflow_address_count() {}

#[test]
fn ipv6_slash_zero_reports_two_to_the_power_of_128() {}

#[test]
fn non_contiguous_ipv4_mask_is_rejected() {}
```

### Compatibility Tests

Compare representative IPv4 calculations with `ipcalc`. Formatting does not need to be identical, but network, mask, wildcard, broadcast, host range, and counts must match.

Check IPv6 behavior against known CIDR test vectors because traditional `ipcalc` versions may not consistently support IPv6.

## 8. CI Quality Checks

Each pull request should run:

```console
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```

Optional checks include minimum supported Rust version testing, coverage, `cargo audit`, dependency license checks, and parser fuzz testing.

## 9. Example IPv4 Output

Command:

```console
$ ipclc 192.168.10.42/24
```

Output:

```text
Address:          192.168.10.42
Address type:     Private
CIDR:             192.168.10.42/24
Netmask:          255.255.255.0
Wildcard:         0.0.0.255
Network:          192.168.10.0
Broadcast:        192.168.10.255
Host range:       192.168.10.1 - 192.168.10.254
Total addresses:  256
Usable hosts:     254
```

## 10. Example IPv6 Output

Command:

```console
$ ipclc 2001:db8:1234:5678::42/64
```

Output:

```text
Address:          2001:db8:1234:5678::42
Expanded address: 2001:0db8:1234:5678:0000:0000:0000:0042
Address type:     Documentation
CIDR:             2001:db8:1234:5678::42/64
Prefix length:    64
Netmask:          ffff:ffff:ffff:ffff::
Network:          2001:db8:1234:5678::
First address:    2001:db8:1234:5678::
Last address:     2001:db8:1234:5678:ffff:ffff:ffff:ffff
Total addresses:  18446744073709551616 (2^64)
Broadcast:        not applicable
```

## 11. Suggested Release Scope

Version `0.1.0` should include:

- IPv4 and IPv6 CIDR parsing
- Separate dotted IPv4 masks
- Network calculations and address classification
- Human-readable and JSON output
- Unit, property-based, integration, snapshot, and cross-platform tests
- CI builds for Linux, macOS, and Windows

Subnet splitting, reverse DNS, prefix aggregation, and overlap detection can be delivered in later releases.
