//! Network calculation API.
//!
//! The implementation currently lives in the crate root to keep the public
//! API compact; these re-exports provide a stable namespace for consumers.
pub use crate::{
    adjacent_network, aggregate_networks, calculate_ipv4, calculate_ipv6, contains,
    networks_overlap, parse_network, reverse_dns, reverse_dns_zone, split_network,
};
pub use crate::{Ipv4NetworkInfo, Ipv6NetworkInfo, NetworkInfo};
