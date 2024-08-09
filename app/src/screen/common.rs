use crate::api::common::Network;

pub fn network_symbol(network: Network) -> String {
    match network {
        Network::Bitcoin => "₿",
        Network::Ethereum => "⟠",
    }
    .to_string()
}
