<div align="center">
  <img height="100" src="assets/logo.png"/>
  <h2> TUI for managing bluetooth on Linux </h2>
  <img src="https://github.com/user-attachments/assets/13498e90-b2d0-43c8-9dd7-8a9de2734f64"/>
</div>

## 💡 Prerequisites

A Linux based OS with [bluez](https://www.bluez.org/) installed.

> [!NOTE]
> You might need to install [nerdfonts](https://www.nerdfonts.com/) for the icons to be displayed correctly.

## 🚀 Installation

### 📥 Binary release

You can download the pre-built binaries from the release page [release page](https://github.com/pythops/bluetui/releases)

### 📦 crates.io

You can install `bluetui` from [crates.io](https://crates.io/crates/bluetui)

```shell
cargo install bluetui
```

### 🐧 Arch Linux

You can install `bluetui` from the [extra repository](https://archlinux.org/packages/extra/x86_64/bluetui/):

```shell
pacman -S bluetui
```

### 🐧 Gentoo

You can install `bluetui` from the [lamdness Gentoo Overlay](https://gpo.zugaina.org/net-wireless/bluetui):
```sh
sudo eselect repository enable lamdness
sudo emaint -r lamdness sync
sudo emerge -av net-wireless/bluetui
```

### 🧰 X-CMD

If you are a user of [x-cmd](https://x-cmd.com), you can run:

```shell
x install bluetui
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

`ctrl+c` or `q`: Quit the app.

### Adapters

`p`: Enable/Disable the pairing.

`o`: Power on/off the adapter.

`d`: Enable/Disable the discovery.

### Paired devices

`u`: Unpair the device.

`Space`: Connect/Disconnect the device.

`t`: Trust/Untrust the device.

`e`: Rename the device.

### New devices

`p`: Pair the device.

## Custom keybindings

Keybindings can be customized in the config file `$HOME/.config/bluetui/config.toml`

```toml
toggle_scanning = "s"

[adapter]
toggle_pairing = "p"
toggle_power = "o"
toggle_discovery = "d"

[paired_device]
unpair = "u"
toggle_connect = " "
toggle_trust = "t"
rename = "e"

[new_device]
pair = "p"
```

## ⚖️ License

GPLv3
