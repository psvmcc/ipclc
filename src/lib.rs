use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use serde::Serialize;
use thiserror::Error;

pub mod network;
pub mod output;

#[derive(Debug, Error)]
pub enum IpclcError {
    #[error("invalid IP address: {0}")]
    InvalidAddress(String),
    #[error("invalid IPv4 netmask: {0}")]
    InvalidNetmask(String),
    #[error("invalid prefix length: {0}")]
    InvalidPrefix(String),
    #[error("address and netmask must use the same IP version")]
    VersionMismatch,
    #[error("the address does not include a CIDR prefix")]
    MissingPrefix,
    #[error("requested subnet prefix must be longer than the network prefix")]
    SplitPrefixTooShort,
    #[error("subnet split would produce too many networks")]
    SplitTooLarge,
    #[error("the prefix is not aligned for a reverse DNS zone")]
    ReverseZoneUnaligned,
    #[error("the network has no {0} adjacent prefix")]
    NoAdjacentNetwork(&'static str),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressType {
    Unspecified,
    Loopback,
    Private,
    Shared,
    Benchmarking,
    Ipv4Mapped,
    UniqueLocal,
    LinkLocal,
    Documentation,
    Multicast,
    Global,
    Reserved,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "version")]
pub enum NetworkInfo {
    #[serde(rename = "4")]
    V4(Ipv4NetworkInfo),
    #[serde(rename = "6")]
    V6(Ipv6NetworkInfo),
}

#[derive(Debug, Clone, Serialize)]
pub struct Ipv4NetworkInfo {
    pub address: Ipv4Addr,
    pub address_type: AddressType,
    pub prefix: u8,
    pub netmask: Ipv4Addr,
    pub wildcard: Ipv4Addr,
    pub network: Ipv4Addr,
    pub broadcast: Ipv4Addr,
    pub first_host: Ipv4Addr,
    pub last_host: Ipv4Addr,
    pub total_addresses: u64,
    pub usable_hosts: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Ipv6NetworkInfo {
    pub address: Ipv6Addr,
    pub expanded_address: String,
    pub address_type: AddressType,
    pub prefix: u8,
    pub netmask: Ipv6Addr,
    pub network: Ipv6Addr,
    pub first_address: Ipv6Addr,
    pub last_address: Ipv6Addr,
    pub total_addresses: String,
}

pub fn parse_network(input: &str, separate_mask: Option<&str>) -> Result<NetworkInfo, IpclcError> {
    if let Some(mask) = separate_mask {
        let ip: Ipv4Addr = input
            .parse()
            .map_err(|_| IpclcError::InvalidAddress(input.into()))?;
        let mask: Ipv4Addr = mask
            .parse()
            .map_err(|_| IpclcError::InvalidNetmask(mask.into()))?;
        let prefix =
            mask_to_prefix(mask).ok_or_else(|| IpclcError::InvalidNetmask(mask.to_string()))?;
        return Ok(NetworkInfo::V4(calculate_ipv4(ip, prefix)?));
    }

    let (address, prefix_text) = input.rsplit_once('/').ok_or(IpclcError::MissingPrefix)?;
    let ip: IpAddr = address
        .parse()
        .map_err(|_| IpclcError::InvalidAddress(address.into()))?;
    let prefix: u8 = match prefix_text.parse() {
        Ok(prefix) => prefix,
        Err(_) if prefix_text.contains('.') && ip.is_ipv4() => {
            let mask: Ipv4Addr = prefix_text
                .parse()
                .map_err(|_| IpclcError::InvalidNetmask(prefix_text.into()))?;
            mask_to_prefix(mask).ok_or_else(|| IpclcError::InvalidNetmask(prefix_text.into()))?
        }
        Err(_) => return Err(IpclcError::InvalidPrefix(prefix_text.into())),
    };
    match ip {
        IpAddr::V4(ip) if prefix <= 32 => Ok(NetworkInfo::V4(calculate_ipv4(ip, prefix)?)),
        IpAddr::V6(ip) if prefix <= 128 => Ok(NetworkInfo::V6(calculate_ipv6(ip, prefix)?)),
        IpAddr::V4(_) => Err(IpclcError::InvalidPrefix(prefix_text.into())),
        IpAddr::V6(_) => Err(IpclcError::InvalidPrefix(prefix_text.into())),
    }
}

pub fn contains(info: &NetworkInfo, candidate: IpAddr) -> Result<bool, IpclcError> {
    match (info, candidate) {
        (NetworkInfo::V4(network), IpAddr::V4(ip)) => {
            let value = u32::from(ip);
            Ok(value >= u32::from(network.network) && value <= u32::from(network.broadcast))
        }
        (NetworkInfo::V6(network), IpAddr::V6(ip)) => {
            let value = u128::from(ip);
            Ok(value >= u128::from(network.first_address)
                && value <= u128::from(network.last_address))
        }
        _ => Err(IpclcError::VersionMismatch),
    }
}

/// Return the fully-qualified reverse-DNS name for an address.
pub fn reverse_dns(address: IpAddr) -> String {
    match address {
        IpAddr::V4(ip) => {
            ip.octets()
                .iter()
                .rev()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(".")
                + ".in-addr.arpa."
        }
        IpAddr::V6(ip) => {
            format!("{:032x}", u128::from(ip))
                .chars()
                .rev()
                .map(|ch| ch.to_string())
                .collect::<Vec<_>>()
                .join(".")
                + ".ip6.arpa."
        }
    }
}

pub fn reverse_dns_zone(info: &NetworkInfo) -> Result<String, IpclcError> {
    match info {
        NetworkInfo::V4(n) => {
            if n.prefix % 8 != 0 {
                return Err(IpclcError::ReverseZoneUnaligned);
            }
            let count = (n.prefix / 8) as usize;
            if count == 0 {
                return Ok("in-addr.arpa.".into());
            }
            Ok(n.network.octets()[..count]
                .iter()
                .rev()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(".")
                + ".in-addr.arpa.")
        }
        NetworkInfo::V6(n) => {
            if n.prefix % 4 != 0 {
                return Err(IpclcError::ReverseZoneUnaligned);
            }
            let hex = format!("{:032x}", u128::from(n.network));
            if n.prefix == 0 {
                return Ok("ip6.arpa.".into());
            }
            Ok(hex[..n.prefix as usize]
                .chars()
                .rev()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(".")
                + ".ip6.arpa.")
        }
    }
}

pub fn adjacent_network(info: &NetworkInfo, next: bool) -> Result<NetworkInfo, IpclcError> {
    match info {
        NetworkInfo::V4(n) => {
            if n.prefix > 32 {
                return Err(IpclcError::InvalidPrefix(n.prefix.to_string()));
            }
            if n.prefix == 0 {
                return Err(IpclcError::NoAdjacentNetwork(if next {
                    "next"
                } else {
                    "previous"
                }));
            }
            let size = 1u32 << (32 - n.prefix);
            let value = u32::from(n.network);
            let adjacent = if next {
                value.checked_add(size)
            } else {
                value.checked_sub(size)
            }
            .ok_or(IpclcError::NoAdjacentNetwork(if next {
                "next"
            } else {
                "previous"
            }))?;
            Ok(NetworkInfo::V4(calculate_ipv4_unchecked(
                Ipv4Addr::from(adjacent),
                n.prefix,
            )))
        }
        NetworkInfo::V6(n) => {
            if n.prefix > 128 {
                return Err(IpclcError::InvalidPrefix(n.prefix.to_string()));
            }
            if n.prefix == 0 {
                return Err(IpclcError::NoAdjacentNetwork(if next {
                    "next"
                } else {
                    "previous"
                }));
            }
            let size = 1u128 << (128 - n.prefix);
            let value = u128::from(n.network);
            let adjacent = if next {
                value.checked_add(size)
            } else {
                value.checked_sub(size)
            }
            .ok_or(IpclcError::NoAdjacentNetwork(if next {
                "next"
            } else {
                "previous"
            }))?;
            Ok(NetworkInfo::V6(calculate_ipv6_unchecked(
                Ipv6Addr::from(adjacent),
                n.prefix,
            )))
        }
    }
}

/// Check whether two prefixes overlap. Networks of different IP versions do
/// not overlap and return `false`.
pub fn networks_overlap(left: &NetworkInfo, right: &NetworkInfo) -> Result<bool, IpclcError> {
    match (left, right) {
        (NetworkInfo::V4(a), NetworkInfo::V4(b)) => Ok(u32::from(a.network)
            <= u32::from(b.broadcast)
            && u32::from(b.network) <= u32::from(a.broadcast)),
        (NetworkInfo::V6(a), NetworkInfo::V6(b)) => Ok(u128::from(a.first_address)
            <= u128::from(b.last_address)
            && u128::from(b.first_address) <= u128::from(a.last_address)),
        _ => Err(IpclcError::VersionMismatch),
    }
}

fn network_sort_key(network: &NetworkInfo) -> (u8, u128) {
    match network {
        NetworkInfo::V4(value) => (value.prefix, u128::from(u32::from(value.network))),
        NetworkInfo::V6(value) => (value.prefix, u128::from(value.network)),
    }
}

fn network_contains_network(parent: &NetworkInfo, child: &NetworkInfo) -> bool {
    match (parent, child) {
        (NetworkInfo::V4(parent), NetworkInfo::V4(child)) => {
            parent.prefix <= child.prefix
                && u32::from(parent.network) <= u32::from(child.network)
                && u32::from(parent.broadcast) >= u32::from(child.broadcast)
        }
        (NetworkInfo::V6(parent), NetworkInfo::V6(child)) => {
            parent.prefix <= child.prefix
                && u128::from(parent.first_address) <= u128::from(child.first_address)
                && u128::from(parent.last_address) >= u128::from(child.last_address)
        }
        _ => false,
    }
}

/// Aggregate adjacent, equally sized prefixes where they form an aligned
/// parent prefix. The operation is repeated until no more merges are possible.
pub fn aggregate_networks(mut networks: Vec<NetworkInfo>) -> Result<Vec<NetworkInfo>, IpclcError> {
    for network in &networks {
        match network {
            NetworkInfo::V4(value) if value.prefix > 32 => {
                return Err(IpclcError::InvalidPrefix(value.prefix.to_string()));
            }
            NetworkInfo::V6(value) if value.prefix > 128 => {
                return Err(IpclcError::InvalidPrefix(value.prefix.to_string()));
            }
            _ => {}
        }
    }
    if networks
        .iter()
        .any(|network| std::mem::discriminant(network) != std::mem::discriminant(&networks[0]))
    {
        return Err(IpclcError::VersionMismatch);
    }
    networks.sort_by_key(network_sort_key);
    let mut normalized = Vec::with_capacity(networks.len());
    for network in networks {
        if !normalized
            .iter()
            .any(|parent| network_contains_network(parent, &network))
        {
            normalized.push(network);
        }
    }
    let mut networks = normalized;
    let mut changed = true;
    while changed {
        changed = false;
        'outer: for left in 0..networks.len() {
            for right in (left + 1)..networks.len() {
                let merged = match (&networks[left], &networks[right]) {
                    (NetworkInfo::V4(a), NetworkInfo::V4(b))
                        if a.prefix == b.prefix && a.prefix > 0 =>
                    {
                        let parent_prefix = a.prefix - 1;
                        let mask = ipv4_mask(parent_prefix);
                        if a.network != b.network
                            && (u32::from(a.network) & mask) == (u32::from(b.network) & mask)
                        {
                            Some(NetworkInfo::V4(calculate_ipv4_unchecked(
                                Ipv4Addr::from(u32::from(a.network) & mask),
                                parent_prefix,
                            )))
                        } else {
                            None
                        }
                    }
                    (NetworkInfo::V6(a), NetworkInfo::V6(b))
                        if a.prefix == b.prefix && a.prefix > 0 =>
                    {
                        let parent_prefix = a.prefix - 1;
                        let mask = ipv6_mask(parent_prefix);
                        if a.network != b.network
                            && (u128::from(a.network) & mask) == (u128::from(b.network) & mask)
                        {
                            Some(NetworkInfo::V6(calculate_ipv6_unchecked(
                                Ipv6Addr::from(u128::from(a.network) & mask),
                                parent_prefix,
                            )))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                if let Some(merged) = merged {
                    networks.remove(right);
                    networks[left] = merged;
                    changed = true;
                    break 'outer;
                }
            }
        }
    }
    networks.sort_by_key(network_sort_key);
    Ok(networks)
}

/// Split a network into child prefixes. A guard prevents accidental allocation
/// of billions of entries when a very broad prefix is supplied.
pub fn split_network(info: &NetworkInfo, new_prefix: u8) -> Result<Vec<NetworkInfo>, IpclcError> {
    const MAX_SUBNETS: u128 = 4096;
    match info {
        NetworkInfo::V4(network) => {
            if new_prefix <= network.prefix || new_prefix > 32 {
                return Err(IpclcError::SplitPrefixTooShort);
            }
            let count = 1u128 << (new_prefix - network.prefix);
            if count > MAX_SUBNETS {
                return Err(IpclcError::SplitTooLarge);
            }
            let step = 1u32 << (32 - new_prefix);
            let start = u32::from(network.network);
            Ok((0..count as u32)
                .map(|index| {
                    NetworkInfo::V4(calculate_ipv4_unchecked(
                        Ipv4Addr::from(start + index * step),
                        new_prefix,
                    ))
                })
                .collect())
        }
        NetworkInfo::V6(network) => {
            if new_prefix <= network.prefix || new_prefix > 128 {
                return Err(IpclcError::SplitPrefixTooShort);
            }
            let difference = new_prefix - network.prefix;
            let count = if difference == 128 {
                u128::MAX
            } else {
                1u128 << difference
            };
            if count > MAX_SUBNETS {
                return Err(IpclcError::SplitTooLarge);
            }
            let step = 1u128 << (128 - new_prefix);
            let start = u128::from(network.network);
            Ok((0..count)
                .map(|index| {
                    NetworkInfo::V6(calculate_ipv6_unchecked(
                        Ipv6Addr::from(start + index * step),
                        new_prefix,
                    ))
                })
                .collect())
        }
    }
}

pub fn calculate_ipv4(address: Ipv4Addr, prefix: u8) -> Result<Ipv4NetworkInfo, IpclcError> {
    if prefix > 32 {
        return Err(IpclcError::InvalidPrefix(prefix.to_string()));
    }
    Ok(calculate_ipv4_unchecked(address, prefix))
}

fn calculate_ipv4_unchecked(address: Ipv4Addr, prefix: u8) -> Ipv4NetworkInfo {
    let mask = ipv4_mask(prefix);
    let address_value = u32::from(address);
    let network = address_value & mask;
    let wildcard = !mask;
    let broadcast = network | wildcard;
    let total = u64::from(wildcard) + 1;
    let (first, last, usable) = if prefix == 32 {
        (network, network, 1)
    } else if prefix == 31 {
        (network, broadcast, 2)
    } else {
        (network + 1, broadcast - 1, total - 2)
    };
    Ipv4NetworkInfo {
        address,
        address_type: classify_ipv4(address),
        prefix,
        netmask: Ipv4Addr::from(mask),
        wildcard: Ipv4Addr::from(wildcard),
        network: Ipv4Addr::from(network),
        broadcast: Ipv4Addr::from(broadcast),
        first_host: Ipv4Addr::from(first),
        last_host: Ipv4Addr::from(last),
        total_addresses: total,
        usable_hosts: usable,
    }
}

pub fn calculate_ipv6(address: Ipv6Addr, prefix: u8) -> Result<Ipv6NetworkInfo, IpclcError> {
    if prefix > 128 {
        return Err(IpclcError::InvalidPrefix(prefix.to_string()));
    }
    Ok(calculate_ipv6_unchecked(address, prefix))
}

fn calculate_ipv6_unchecked(address: Ipv6Addr, prefix: u8) -> Ipv6NetworkInfo {
    let mask = ipv6_mask(prefix);
    let network = u128::from(address) & mask;
    let last = network | !mask;
    Ipv6NetworkInfo {
        address,
        expanded_address: address
            .segments()
            .iter()
            .map(|part| format!("{part:04x}"))
            .collect::<Vec<_>>()
            .join(":"),
        address_type: classify_ipv6(address),
        prefix,
        netmask: Ipv6Addr::from(mask),
        network: Ipv6Addr::from(network),
        first_address: Ipv6Addr::from(network),
        last_address: Ipv6Addr::from(last),
        total_addresses: if prefix == 0 {
            "340282366920938463463374607431768211456".into()
        } else {
            (1u128 << (128 - prefix)).to_string()
        },
    }
}

fn mask_to_prefix(mask: Ipv4Addr) -> Option<u8> {
    let value = u32::from(mask);
    let prefix = value.leading_ones() as u8;
    if value == ipv4_mask(prefix) {
        Some(prefix)
    } else {
        None
    }
}

fn classify_ipv4(ip: Ipv4Addr) -> AddressType {
    let o = ip.octets();
    if ip.is_unspecified() {
        AddressType::Unspecified
    } else if ip.is_loopback() {
        AddressType::Loopback
    } else if o[0] == 100 && (64..=127).contains(&o[1]) {
        AddressType::Shared
    } else if o[0] == 198 && (o[1] == 18 || o[1] == 19) {
        AddressType::Benchmarking
    } else if ip.is_private() {
        AddressType::Private
    } else if ip.is_link_local() {
        AddressType::LinkLocal
    } else if ip.is_multicast() {
        AddressType::Multicast
    } else if (o[0] == 192 && o[1] == 0 && o[2] == 2)
        || (o[0] == 198 && o[1] == 51 && o[2] == 100)
        || (o[0] == 203 && o[1] == 0 && o[2] == 113)
    {
        AddressType::Documentation
    } else if o[0] == 0 || o[0] >= 240 || (o[0] == 192 && o[1] == 0 && o[2] == 0) {
        AddressType::Reserved
    } else {
        AddressType::Global
    }
}

fn classify_ipv6(ip: Ipv6Addr) -> AddressType {
    if ip.is_unspecified() {
        AddressType::Unspecified
    } else if ip.is_loopback() {
        AddressType::Loopback
    } else if (u128::from(ip) >> 32) == 0xffff {
        AddressType::Ipv4Mapped
    } else if ip.is_unique_local() {
        AddressType::UniqueLocal
    } else if ip.is_unicast_link_local() {
        AddressType::LinkLocal
    } else if ip.is_multicast() {
        AddressType::Multicast
    } else if (u128::from(ip) >> 96) == 0x20010db8 {
        AddressType::Documentation
    } else {
        AddressType::Global
    }
}

fn ipv4_mask(prefix: u8) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    }
}

fn ipv6_mask(prefix: u8) -> u128 {
    if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv4_calculation() {
        let n = calculate_ipv4("192.168.10.42".parse().unwrap(), 24).unwrap();
        assert_eq!(n.network, "192.168.10.0".parse::<Ipv4Addr>().unwrap());
        assert_eq!(n.broadcast, "192.168.10.255".parse::<Ipv4Addr>().unwrap());
        assert_eq!(n.usable_hosts, 254);
    }

    #[test]
    fn ipv6_calculation() {
        let n = calculate_ipv6("2001:db8::42".parse().unwrap(), 64).unwrap();
        assert_eq!(n.network, "2001:db8::".parse::<Ipv6Addr>().unwrap());
        assert_eq!(n.total_addresses, "18446744073709551616");
    }

    #[test]
    fn rejects_non_contiguous_mask() {
        assert!(parse_network("192.168.1.1", Some("255.0.255.0")).is_err());
    }
}
