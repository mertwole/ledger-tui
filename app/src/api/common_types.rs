#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Network {
    Bitcoin,
    Ethereum,
}

pub struct NetworkInfo {
    pub name: String,
    #[allow(dead_code)]
    pub symbol: String,
}

impl Network {
    pub fn get_info(&self) -> NetworkInfo {
        match self {
            Self::Bitcoin => NetworkInfo {
                name: "Bitcoin".into(),
                symbol: "BTC".into(),
            },
            Self::Ethereum => NetworkInfo {
                name: "Ethereum".into(),
                symbol: "ETH".into(),
            },
        }
    }
}

// TODO: Don't allow to construct it directly
// And allow mocks to substitute `MockAccount` instead of `Account`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Account {
    pub public_key: String,
}

impl Account {
    pub fn get_info(&self) -> AccountInfo {
        AccountInfo {
            public_key: self.public_key.clone(),
        }
    }
}

pub struct AccountInfo {
    #[allow(dead_code)]
    /// Public key of account in encoding native for network,
    pub public_key: String,
}
