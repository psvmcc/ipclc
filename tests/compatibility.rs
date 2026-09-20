//! Compatibility vectors shared with common ipcalc implementations.
//! These test values rather than formatting, so they remain portable.
use ipclc::{calculate_ipv4, calculate_ipv6};

#[test]
fn ipcalc_ipv4_vector() {
    let n = calculate_ipv4("192.168.1.42".parse().unwrap(), 24).unwrap();
    assert_eq!(n.network.to_string(), "192.168.1.0");
    assert_eq!(n.netmask.to_string(), "255.255.255.0");
    assert_eq!(n.broadcast.to_string(), "192.168.1.255");
}

#[test]
fn ipcalc_ipv6_vector() {
    let n = calculate_ipv6("2001:db8::1".parse().unwrap(), 64).unwrap();
    assert_eq!(n.network.to_string(), "2001:db8::");
    assert_eq!(n.netmask.to_string(), "ffff:ffff:ffff:ffff::");
}
