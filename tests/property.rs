use std::net::{Ipv4Addr, Ipv6Addr};

use ipclc::{calculate_ipv4, calculate_ipv6};
use proptest::prelude::*;

proptest! {
    #[test]
    fn ipv4_address_is_inside_calculated_network(raw in any::<u32>(), prefix in 0u8..=32) {
        let info = calculate_ipv4(Ipv4Addr::from(raw), prefix).unwrap();
        prop_assert!(u32::from(info.network) <= raw);
        prop_assert!(raw <= u32::from(info.broadcast));
        prop_assert_eq!(u32::from(info.network) & u32::from(info.wildcard), 0);
    }

    #[test]
    fn ipv6_address_is_inside_calculated_prefix(raw in any::<u128>(), prefix in 0u8..=128) {
        let info = calculate_ipv6(Ipv6Addr::from(raw), prefix).unwrap();
        prop_assert!(u128::from(info.first_address) <= raw);
        prop_assert!(raw <= u128::from(info.last_address));
    }
}
