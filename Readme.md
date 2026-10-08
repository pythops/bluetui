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

```
$ bluetui
```

You can run `impala` with options:

```
$ bluetui --help
Options:
  -c, --config <path>  config file path. Default ~/.config/bluetui/config.toml
      --ascii          Ascii display
  -h, --help           Print help
  -V, --version        Print version
```

## Config

Keybindings can be customized in the default config file location `$HOME/.config/bluetui/config.toml` or from a custom path with `-c`

```toml
ascii = false # ascii display (false = do not show icons)

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
```

## 🤝 Contributing

- Strict No LLM.
- Only submit a pull request after having a prior issue or discussion.
- Keep PRs small and focused.

## 🎁 Note

If you like `bluetui` and you are looking for a TUI to manage WiFi, checkout out [impala](https://github.com/pythops/impala)

## ⚖️ License

GPLv3

## ✍️ Credits

Bluetui logo: [Marco Bulgarelli](https://github.com/Bugg4)
