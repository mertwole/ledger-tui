use ledger_lib::{
    Device as LedgerDevice, Filters, LedgerHandle, LedgerInfo, LedgerProvider, Transport,
    DEFAULT_TIMEOUT,
};

pub struct Device {
    provider: LedgerProvider,

    device: LedgerInfo,
    handle: Option<LedgerHandle>,
}

impl Device {
    pub async fn discover() -> Option<Device> {
        let mut provider = LedgerProvider::init().await;
        let mut devices = provider.list(Filters::Any).await.unwrap();

        if devices.is_empty() {
            return None;
        }

        let device = devices.remove(0);
        let handle = provider.connect(device.clone()).await.unwrap();

        Some(Self {
            provider,
            device,
            handle: Some(handle),
        })
    }

    pub async fn get_info(&mut self) -> String {
        let info = loop {
            let Some(handle) = self.handle.as_mut() else {
                self.connect().await;
                continue;
            };

            let info = handle.app_info(DEFAULT_TIMEOUT).await;

            if let Ok(info) = info {
                break info;
            } else {
                self.connect().await;
            }
        };

        format!("{} v{}", info.name, info.version)
    }

    async fn connect(&mut self) {
        self.handle = None;
        self.handle = self.provider.connect(self.device.clone()).await.ok();
    }
}
