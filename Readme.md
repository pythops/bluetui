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

```sh
emerge net-wireless/bluetui
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

`f`: Favorite/Unfavorite the device.

`e`: Rename the device.

### New devices

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
toggle_favorite = "f"
rename = "e"

[navigation]
up = "k"
down = "j"
left = "h"
right = "l"
quit = "q"
select = " "  # Space key for connect/disconnect/pair

[colors]
# Colors can be specified as named colors or hex values (#RRGGBB)
# Named colors: black, red, green, yellow, blue, magenta, cyan, gray, dark_gray,
#               light_red, light_green, light_yellow, light_blue, light_magenta,
#               light_cyan, white, reset

# Border color when a section is focused
focused_border = "green"

# Border color when a section is not focused
unfocused_border = "reset"

# Header text color in focused tables
focused_header = "yellow"

# Background color for selected items
highlight_bg = "dark_gray"

# Text color for selected items
highlight_fg = "white"

# Color for informational messages
info = "green"

# Color for warning messages
warning = "yellow"

# Color for error messages
error = "red"

# Color for the scanning spinner
spinner = "blue"

# Color for the help text at the bottom of the window
help_text = "blue"
```

## Contributing

- No AI slop.
- Only submit a pull request after having a prior issue or discussion.
- Keep PRs small and focused.

## 🎁 Note

If you like `bluetui` and you are looking for a TUI to manage WiFi, checkout out [impala](https://github.com/pythops/impala)

## ⚖️ License

GPLv3

## ✍️ Credits

Bluetui logo: [Marco Bulgarelli](https://github.com/Bugg4)
