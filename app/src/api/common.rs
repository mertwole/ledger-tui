#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Account {
    pub pk: String,
}

impl Account {
    pub fn get_info(&self) -> AccountInfo {
        AccountInfo {
            pk: self.pk.clone(),
        }
    }
}

pub struct AccountInfo {
    #[allow(dead_code)]
    /// Public key of account in encoding native for network,
    pub pk: String,
}
