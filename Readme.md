<div align="center">
  <img height="100" src="assets/logo.png"/>
  <h2> TUI to manage bluetooth devices </h2>
  <img src="https://github.com/pythops/bluetui/assets/57548585/e4d475d8-9983-4428-9642-03016a8ed352"/>
</div>

## 💡 Prerequisites

A Linux based OS with [bluez](https://www.bluez.org/) installed.

## 🚀 Installation

### 📥 Binary release

You can download the pre-built binaries from the release page [release page](https://github.com/pythops/bluetui/releases)

### 📦 crates.io

You can install `bluetui` from [crates.io](https://crates.io/crates/bluetui)

```shell
cargo install bluetui
```

### ⚒️ Build from source

Run the following command:

```shell
git clone https://github.com/pythops/bluetui
cd bluetui
cargo build --release
```

This will produce an executable file at `target/release/bluetui` that you can copy to a directory in your `$PATH`.

## 🪄 Usage

### Global

`Tab`: Switch between different sections.

`j` or `Down` : Scroll down.

`k` or `Up`: Scroll up.

`s`: Start/Stop scanning.

`?`: Show help.

`esc`: Dismiss the help pop-up.

`q`: Quit the app.

### Adpters

`p`: Enable/Disable the pairing.

`o`: Power on/off the adapter.

`d`: Enable/Disable the discovery.

### Paired devices

`u`: Unpair the device.

`Space`: Connect/Disconnect the device.

`t`: Trust/Untrust the device.

### New devices

`p`: Pair the device.

## 🗹 Todo

- [ ] handle authentication

## ⚖️ License

GPLv3
