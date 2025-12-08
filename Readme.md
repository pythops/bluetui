<div align="center">
  <img height="125" src="assets/bluetui-logo-anim.svg"/>
  <h2> TUI for managing bluetooth on Linux </h2>
  <img src="https://github.com/user-attachments/assets/f937535d-5675-4427-b347-8086c8830e23"/>
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

`Tab` or `l`: Scroll down between different sections.

`shift+Tab` or `h`: Scroll up between different sections.

`j` or `Down` : Scroll down.

`k` or `Up`: Scroll up.

`s`: Start/Stop scanning.

`ctrl+c` or `q`: Quit the app. (Note: `<Esc>` can also quit if `esc_quit = true` is set in config)

### Adapters

`p`: Enable/Disable the pairing.

`o`: Power on/off the adapter.

`d`: Enable/Disable the discovery.

### Paired devices

`u`: Unpair the device.

`Space or Enter`: Connect/Disconnect the device.

`t`: Trust/Untrust the device.

`e`: Rename the device.

### New devices

`/`: Fuzzy search devices. In search mode:
  - All devices shown initially (start typing to filter)
  - Type any characters to filter devices in real-time (including 'j' and 'k')
  - `↑/↓`: Navigate through results (arrow keys only)
  - `Enter` or `Space`: Pair with selected device
  - `Esc`: Exit search with selected device highlighted

`Space or Enter`: Pair the device.

## Config

Keybindings can be customized in the default config file location `$HOME/.config/bluetui/config.toml` or from a custom path with `-c`

```toml
# Possible values: "Legacy", "Start", "End", "Center", "SpaceAround", "SpaceBetween"
layout = "SpaceAround"

# Window width
# Possible values: "auto" or a positive integer
width = "auto"

toggle_scanning = "s"
esc_quit = false  # Set to true to enable Esc key to quit the app

[adapter]
toggle_pairing = "p"
toggle_power = "o"
toggle_discovery = "d"

 [paired_device]
 unpair = "u"
 toggle_trust = "t"
 rename = "e"

 [new_devices]
 search = "/"
```

## 🎁 Note

If you like `bluetui` and you are looking for a TUI to manage WiFi, checkout out [impala](https://github.com/pythops/impala)

## ⚖️ License

GPLv3

## ✍️ Credits

Bluetui logo: [Marco Bulgarelli](https://github.com/Bugg4)
