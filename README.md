# Ledger TUI

Ledger-tui is an application for managing crypto assets on a [ledger](https://www.ledger.com/) device.

> [!NOTE]
> This software is not guaranteed to work properly. Any usage can lead to funds loss and `ledger` device corruption.

## Build

### Ubuntu 24.0

Install dependencies:

- `libssl-dev`
- `libdbus-1-dev`
- `pkg-config`
- `build-essential`
- `libusb-1.0-0-dev`

Instal rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Build application:

```bash
cargo build --release
```

## License

[MIT license](https://github.com/mertwole/ledger-tui/blob/main/LICENSE)