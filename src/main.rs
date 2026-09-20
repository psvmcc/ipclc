use std::io::IsTerminal;
use std::net::IpAddr;

use clap::{ArgGroup, CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, Shell};
use ipclc::{
    adjacent_network, aggregate_networks, contains, networks_overlap, parse_network, reverse_dns,
    reverse_dns_zone, split_network, NetworkInfo,
};

#[derive(Clone, Debug, ValueEnum)]
enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Parser)]
#[command(
    name = "ipclc",
    version,
    about = "IPv4 and IPv6 network calculator",
    group(ArgGroup::new("operation").args([
        "json", "short", "contains", "split", "reverse_dns", "reverse_zone", "overlaps",
        "aggregate", "neighbor", "completions"
    ]).multiple(false))
)]
struct Cli {
    /// Address in CIDR notation, for example 192.168.1.10/24
    #[arg(required_unless_present = "completions")]
    address: Option<String>,
    /// Optional dotted-decimal IPv4 netmask
    netmask: Option<String>,
    /// Print machine-readable JSON
    #[arg(short, long)]
    json: bool,
    /// Print a shortened result
    #[arg(short, long)]
    short: bool,
    /// Set color mode: auto, always, or never
    #[arg(long, value_enum, default_value_t = ColorMode::Auto)]
    color: ColorMode,
    /// Disable colored output
    #[arg(long)]
    no_color: bool,
    /// Check whether an IP belongs to the supplied network
    #[arg(long)]
    contains: Option<String>,
    /// Split the network into child prefixes
    #[arg(long)]
    split: Option<u8>,
    /// Print the reverse DNS name for the supplied address
    #[arg(long)]
    reverse_dns: bool,
    /// Print the reverse DNS zone for the network prefix
    #[arg(long)]
    reverse_zone: bool,
    /// Compare this network with another network
    #[arg(long)]
    overlaps: Option<String>,
    /// Aggregate comma-separated networks
    #[arg(long)]
    aggregate: Option<String>,
    /// Print the adjacent network: previous or next
    #[arg(long, value_parser = ["previous", "next"])]
    neighbor: Option<String>,
    /// Generate shell completions and write them to stdout
    #[arg(long, value_enum)]
    completions: Option<Shell>,
}

fn main() {
    let cli = Cli::parse();
    if let Some(shell) = cli.completions {
        let mut command = Cli::command();
        generate(shell, &mut command, "ipclc", &mut std::io::stdout());
        return;
    }
    let address = match cli.address {
        Some(address) => address,
        None => {
            eprintln!("error: an address is required unless --completions is used");
            std::process::exit(2);
        }
    };
    let info = match parse_network(&address, cli.netmask.as_deref()) {
        Ok(info) => info,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    };

    if let Some(candidate) = cli.contains {
        let candidate: IpAddr = match candidate.parse() {
            Ok(value) => value,
            Err(_) => {
                eprintln!("error: invalid IP address: {candidate}");
                std::process::exit(2);
            }
        };
        match contains(&info, candidate) {
            Ok(result) => println!("{}", if result { "yes" } else { "no" }),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(2);
            }
        }
        return;
    }

    if cli.reverse_dns {
        let address = match &info {
            NetworkInfo::V4(network) => IpAddr::V4(network.address),
            NetworkInfo::V6(network) => IpAddr::V6(network.address),
        };
        println!("{}", reverse_dns(address));
        return;
    }

    if cli.reverse_zone {
        match reverse_dns_zone(&info) {
            Ok(zone) => println!("{zone}"),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(2);
            }
        }
        return;
    }

    if let Some(other) = cli.overlaps {
        let other = match parse_network(&other, None) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(2);
            }
        };
        match networks_overlap(&info, &other) {
            Ok(result) => println!("{}", if result { "yes" } else { "no" }),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(2);
            }
        }
        return;
    }

    if let Some(networks) = cli.aggregate {
        let mut values = vec![info];
        for input in networks.split(',') {
            match parse_network(input.trim(), None) {
                Ok(value) => values.push(value),
                Err(error) => {
                    eprintln!("error: {error}");
                    std::process::exit(2);
                }
            }
        }
        match aggregate_networks(values) {
            Ok(values) => {
                for value in values {
                    println!("{}", cidr(&value));
                }
            }
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(2);
            }
        }
        return;
    }

    if let Some(direction) = cli.neighbor {
        match adjacent_network(&info, direction == "next") {
            Ok(network) => println!("{}", cidr(&network)),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(2);
            }
        }
        return;
    }

    if let Some(prefix) = cli.split {
        match split_network(&info, prefix) {
            Ok(networks) => {
                for network in networks {
                    println!("{}", cidr(&network));
                }
            }
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(2);
            }
        }
        return;
    }

    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&info)
                .expect("serializing network information cannot fail")
        );
    } else {
        let terminal = std::io::stdout().is_terminal();
        let use_color = !cli.no_color
            && match cli.color {
                ColorMode::Always => true,
                ColorMode::Never => false,
                ColorMode::Auto => terminal,
            };
        print_text(&info, cli.short, use_color);
    }
}

fn cidr(info: &NetworkInfo) -> String {
    match info {
        NetworkInfo::V4(network) => format!("{}/{}", network.network, network.prefix),
        NetworkInfo::V6(network) => format!("{}/{}", network.network, network.prefix),
    }
}

fn print_text(info: &NetworkInfo, short: bool, use_color: bool) {
    match info {
        NetworkInfo::V4(n) => {
            if short {
                println!("{}/{}", n.network, n.prefix);
                return;
            }
            print_field("Address:", &n.address.to_string(), use_color, 32);
            print_field(
                "Address type:",
                &type_name(&n.address_type, false),
                use_color,
                36,
            );
            print_field(
                "CIDR:",
                &format!("{}/{}", n.address, n.prefix),
                use_color,
                33,
            );
            print_field("Netmask:", &n.netmask.to_string(), use_color, 35);
            print_field("Wildcard:", &n.wildcard.to_string(), use_color, 35);
            print_field("Network:", &n.network.to_string(), use_color, 33);
            print_field("Broadcast:", &n.broadcast.to_string(), use_color, 31);
            print_field(
                "Host range:",
                &format!("{} - {}", n.first_host, n.last_host),
                use_color,
                32,
            );
            print_field(
                "Total addresses:",
                &format!("{} (2^{})", n.total_addresses, 32 - n.prefix),
                use_color,
                33,
            );
            print_field("Usable hosts:", &n.usable_hosts.to_string(), use_color, 33);
        }
        NetworkInfo::V6(n) => {
            if short {
                println!("{}/{}", n.network, n.prefix);
                return;
            }
            print_field(
                "Address:",
                &format!("{} ({})", n.address, expanded(n.address)),
                use_color,
                32,
            );
            print_field(
                "Address type:",
                &type_name(&n.address_type, false),
                use_color,
                36,
            );
            print_field(
                "CIDR:",
                &format!(
                    "{}/{} ({}/{})",
                    n.address,
                    n.prefix,
                    expanded(n.address),
                    n.prefix
                ),
                use_color,
                33,
            );
            print_field("Prefix length:", &n.prefix.to_string(), use_color, 33);
            print_field("Netmask:", &n.netmask.to_string(), use_color, 35);
            print_field(
                "Network:",
                &format!("{} ({})", n.network, expanded(n.network)),
                use_color,
                33,
            );
            print_field(
                "First address:",
                &format!("{} ({})", n.first_address, expanded(n.first_address)),
                use_color,
                32,
            );
            print_field(
                "Last address:",
                &format!("{} ({})", n.last_address, expanded(n.last_address)),
                use_color,
                32,
            );
            print_field(
                "Total addresses:",
                &format!("{} (2^{})", n.total_addresses, 128 - n.prefix),
                use_color,
                33,
            );
        }
    }
}

fn type_name(address_type: &ipclc::AddressType, use_color: bool) -> String {
    let name = format!("{address_type:?}");
    if use_color {
        format!("\x1b[36m{name}\x1b[0m")
    } else {
        name
    }
}

fn print_field(label: &str, value: &str, use_color: bool, value_color: u8) {
    let label = format!("{label:<18}");
    if use_color {
        println!("\x1b[90m{label}\x1b[0m\x1b[{value_color}m{value}\x1b[0m");
    } else {
        println!("{label}{value}");
    }
}

fn expanded(address: std::net::Ipv6Addr) -> String {
    address
        .segments()
        .iter()
        .map(|segment| format!("{segment:04x}"))
        .collect::<Vec<_>>()
        .join(":")
}
