use ipclc::{
    adjacent_network, aggregate_networks, calculate_ipv4, calculate_ipv6, contains,
    networks_overlap, reverse_dns_zone, AddressType, NetworkInfo,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[test]
fn detects_overlapping_prefixes() {
    let left = NetworkInfo::V4(calculate_ipv4("10.0.0.1".parse().unwrap(), 24).unwrap());
    let right = NetworkInfo::V4(calculate_ipv4("10.0.0.200".parse().unwrap(), 25).unwrap());
    assert!(networks_overlap(&left, &right).unwrap());
}

#[test]
fn aggregates_adjacent_prefixes() {
    let first = NetworkInfo::V4(calculate_ipv4("192.168.1.0".parse().unwrap(), 25).unwrap());
    let second = NetworkInfo::V4(calculate_ipv4("192.168.1.128".parse().unwrap(), 25).unwrap());
    let result = aggregate_networks(vec![first, second]).unwrap();
    assert_eq!(result.len(), 1);
    match &result[0] {
        NetworkInfo::V4(network) => assert_eq!(network.prefix, 24),
        NetworkInfo::V6(_) => panic!("expected IPv4 result"),
    }
}

#[test]
fn aggregates_two_slash_one_prefixes_into_slash_zero() {
    let first = NetworkInfo::V4(calculate_ipv4("0.0.0.0".parse().unwrap(), 1).unwrap());
    let second = NetworkInfo::V4(calculate_ipv4("128.0.0.0".parse().unwrap(), 1).unwrap());
    let result = aggregate_networks(vec![first, second]).unwrap();
    match &result[0] {
        NetworkInfo::V4(network) => assert_eq!(network.prefix, 0),
        NetworkInfo::V6(_) => panic!("expected IPv4 result"),
    }
}

#[test]
fn slash_zero_has_no_adjacent_network() {
    let v4 = NetworkInfo::V4(calculate_ipv4(Ipv4Addr::UNSPECIFIED, 0).unwrap());
    let v6 = NetworkInfo::V6(calculate_ipv6(Ipv6Addr::UNSPECIFIED, 0).unwrap());
    assert!(adjacent_network(&v4, true).is_err());
    assert!(adjacent_network(&v6, false).is_err());
}

#[test]
fn detects_ipv4_mapped_ipv6() {
    let result = calculate_ipv6("::ffff:192.0.2.1".parse().unwrap(), 128).unwrap();
    assert!(matches!(result.address_type, AddressType::Ipv4Mapped));
}

#[test]
fn reverse_dns_root_zones_have_no_leading_dot() {
    let v4 = NetworkInfo::V4(calculate_ipv4(Ipv4Addr::UNSPECIFIED, 0).unwrap());
    let v6 = NetworkInfo::V6(calculate_ipv6(Ipv6Addr::UNSPECIFIED, 0).unwrap());
    assert_eq!(reverse_dns_zone(&v4).unwrap(), "in-addr.arpa.");
    assert_eq!(reverse_dns_zone(&v6).unwrap(), "ip6.arpa.");
}

#[test]
fn mismatched_address_family_is_an_error() {
    let v4 = NetworkInfo::V4(calculate_ipv4(Ipv4Addr::UNSPECIFIED, 0).unwrap());
    assert!(contains(&v4, IpAddr::V6(Ipv6Addr::UNSPECIFIED)).is_err());
}

#[test]
fn public_calculators_reject_invalid_prefixes() {
    assert!(calculate_ipv4(Ipv4Addr::LOCALHOST, 33).is_err());
    assert!(calculate_ipv6(Ipv6Addr::LOCALHOST, 129).is_err());
}

#[test]
fn aggregation_removes_nested_and_duplicate_prefixes() {
    let parent = NetworkInfo::V4(calculate_ipv4("10.0.0.0".parse().unwrap(), 8).unwrap());
    let child = NetworkInfo::V4(calculate_ipv4("10.0.0.0".parse().unwrap(), 9).unwrap());
    let duplicate = parent.clone();
    let result = aggregate_networks(vec![child, duplicate, parent]).unwrap();
    assert_eq!(result.len(), 1);
    match &result[0] {
        NetworkInfo::V4(network) => assert_eq!(network.prefix, 8),
        NetworkInfo::V6(_) => panic!("expected IPv4 result"),
    }
}

#[test]
fn aggregation_rejects_mixed_address_families() {
    let v4 = NetworkInfo::V4(calculate_ipv4(Ipv4Addr::UNSPECIFIED, 0).unwrap());
    let v6 = NetworkInfo::V6(calculate_ipv6(Ipv6Addr::UNSPECIFIED, 0).unwrap());
    assert!(aggregate_networks(vec![v4, v6]).is_err());
}

#[test]
fn overlap_rejects_mixed_address_families() {
    let v4 = NetworkInfo::V4(calculate_ipv4(Ipv4Addr::UNSPECIFIED, 0).unwrap());
    let v6 = NetworkInfo::V6(calculate_ipv6(Ipv6Addr::UNSPECIFIED, 0).unwrap());
    assert!(networks_overlap(&v4, &v6).is_err());
}

#[test]
fn classifies_reserved_ipv4_ranges() {
    for address in ["0.0.0.1", "192.0.0.1", "240.0.0.1", "255.255.255.255"] {
        let result = calculate_ipv4(address.parse().unwrap(), 32).unwrap();
        assert!(matches!(result.address_type, AddressType::Reserved));
    }
}
