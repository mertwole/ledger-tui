use std::{
    collections::HashMap, fs::read_to_string, io::stdout, marker::PhantomData, time::Duration,
};

use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        ExecutableCommand, event,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use toml::Table;

use crate::{
    api::{
        blockchain_monitoring::{
            BlockchainMonitoringApi, BlockchainMonitoringApiT,
            Config as BlockchainMonitoringApiConfig, NetworkApiConfig,
            cache::Cache as BlockchainMonitoringApiCache, mock::BlockchainMonitoringApiMock,
        },
        cache_utils::ModePlan,
        coin_price::{
            CoinPriceApi, CoinPriceApiT, cache::Cache as CoinPriceApiCache, mock::CoinPriceApiMock,
        },
        common_types::{Account, Network},
        ledger::{
            Device, DeviceInfo, LedgerApi, LedgerApiT, cache::Cache as LedgerApiCache,
            mock::LedgerApiMock,
        },
        storage::{StorageApi, StorageApiT, mock::StorageApiMock},
    },
    screen::{OutgoingMessage, Screen, ScreenName, resources::Resources},
};

pub struct App {
    screens: Vec<ScreenName>,
}

type DeviceAccountsList = Vec<(Network, Vec<Account>)>;

// TODO: Add macro to automatically break this registry into sub-registries designated for specific Screens.
pub(crate) struct StateRegistry {
    pub active_device: Option<(Device, DeviceInfo)>,
    pub device_accounts: Option<DeviceAccountsList>,
    pub selected_account: Option<(Network, Account)>,
    _phantom: PhantomData<()>,
}

pub(crate) struct ApiRegistry<L, C, M, S>
where
    L: LedgerApiT,
    C: CoinPriceApiT,
    M: BlockchainMonitoringApiT,
    S: StorageApiT,
{
    pub ledger_api: Option<L>,
    pub coin_price_api: Option<C>,
    pub blockchain_monitoring_api: Option<M>,
    pub storage_api: Option<S>,
    _phantom: PhantomData<()>,
}

impl StateRegistry {
    fn new() -> StateRegistry {
        StateRegistry {
            active_device: Default::default(),
            device_accounts: Default::default(),
            selected_account: None,
            _phantom: PhantomData,
        }
    }
}

impl App {
    pub async fn new() -> Self {
        Self {
            screens: vec![ScreenName::Portfolio, ScreenName::DeviceSelection],
        }
    }

    pub async fn run(mut self) {
        stdout().execute(EnterAlternateScreen).unwrap();
        enable_raw_mode().unwrap();
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
        terminal.clear().unwrap();

        self.main_loop(terminal).await;

        stdout().execute(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }

    async fn main_loop<B: Backend>(&mut self, mut terminal: Terminal<B>) {
        let mut state = StateRegistry::new();

        let mut api_registry = {
            let ledger_api = LedgerApiMock::new(4, 4);
            let _ledger_api = LedgerApi::new().await;
            let mut ledger_api = LedgerApiCache::new(ledger_api).await;
            ledger_api.set_all_modes(ModePlan::Transparent).await;

            let _coin_price_api = CoinPriceApiMock::new();
            let coin_price_api = CoinPriceApi::new("https://data-api.binance.vision");
            let mut coin_price_api = CoinPriceApiCache::new(coin_price_api).await;
            coin_price_api
                .set_all_modes(ModePlan::Slow(Duration::from_secs(0)))
                .await;
            let mut coin_price_api = CoinPriceApiCache::new(coin_price_api).await;
            coin_price_api
                .set_all_modes(ModePlan::TimedOut(Duration::from_secs(3)))
                .await;

            let config = load_blockchain_monitoring_api_config();
            let _blockchain_monitoring_api = BlockchainMonitoringApi::new(config).await;
            let blockchain_monitoring_api = BlockchainMonitoringApiMock::new(4);
            let mut blockchain_monitoring_api =
                BlockchainMonitoringApiCache::new(blockchain_monitoring_api).await;
            blockchain_monitoring_api
                .set_all_modes(ModePlan::TimedOut(Duration::from_secs(3)))
                .await;

            let _storage_api = StorageApi::new("./data".into());
            let storage_api = StorageApiMock::new();

            ApiRegistry {
                ledger_api: Some(ledger_api),
                coin_price_api: Some(coin_price_api),
                blockchain_monitoring_api: Some(blockchain_monitoring_api),
                storage_api: Some(storage_api),
                _phantom: PhantomData,
            }
        };

        loop {
            let screen = self
                .screens
                .last()
                .expect("At least one screen should be present");

            let (new_api_registry, new_state, msg) =
                Self::screen_loop(*screen, state, api_registry, &mut terminal).await;

            api_registry = new_api_registry;
            state = new_state;

            match msg {
                OutgoingMessage::Exit => {
                    return;
                }
                OutgoingMessage::Back => {
                    if self.screens.pop().is_none() {
                        return;
                    }
                }
                OutgoingMessage::SwitchScreen(new_screen) => {
                    self.screens.push(new_screen);
                }
            }
        }
    }

    async fn screen_loop<
        B: Backend,
        L: LedgerApiT,
        C: CoinPriceApiT,
        M: BlockchainMonitoringApiT,
        S: StorageApiT,
    >(
        screen: ScreenName,
        state: StateRegistry,
        api_registry: ApiRegistry<L, C, M, S>,
        terminal: &mut Terminal<B>,
    ) -> (ApiRegistry<L, C, M, S>, StateRegistry, OutgoingMessage) {
        let mut screen = Screen::new(screen, state, api_registry);

        let resources = Resources::default();

        loop {
            terminal
                .draw(|frame| screen.render(frame, &resources))
                .unwrap();

            let event = event::poll(Duration::ZERO)
                .unwrap()
                .then(|| event::read().unwrap());

            let msg = screen.tick(event).await;

            if let Some(msg) = msg {
                let (state, api_registry) = screen.deconstruct().await;
                return (api_registry, state, msg);
            }
        }
    }
}

fn load_blockchain_monitoring_api_config() -> BlockchainMonitoringApiConfig {
    let config =
        read_to_string("NetworkApiConfig.toml").expect("Network api config file is not found");
    let config = config
        .parse::<Table>()
        .expect("Failed to parse NetworkApiConfig.toml");

    let mut network_configs = HashMap::new();
    for (network, network_config) in config {
        let network = match network.as_str() {
            "ethereum" => Network::Ethereum,
            "bitcoin" => Network::Bitcoin,
            str => panic!("Invalid network name found: {}", str),
        };

        let config = network_config
            .as_table()
            .expect("Wrong NetworkApiConfig.toml format")
            .to_string();
        let network_config: NetworkApiConfig =
            toml::from_str(&config).expect("Wrong NetworkApiConfig.toml format");

        network_configs.insert(network, network_config);
    }

    BlockchainMonitoringApiConfig { network_configs }
}
