[![bato](https://img.shields.io/github/actions/workflow/status/doums/bato/test.yml?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=github&style=for-the-badge)](https://github.com/doums/bato/actions?query=workflow%3Atest)
[![bato](https://img.shields.io/aur/version/bato?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=arch-linux&style=for-the-badge)](https://aur.archlinux.org/packages/bato/)

## bato

A program to send **bat**tery level n**o**tifications

![bato](https://github.com/doums/bato/blob/master/img/bato.png)

[features](#features) - [prerequisite](#prerequisite) - [install](#install) - [configuration](#configuration) - [usage](#usage) - [license](#license)

### Features

Tiny configuration in toml.

Desktop notification:

- level full
- level low
- level critical
- charging
- discharging

### Prerequisite

A desktop notification server, like [Dunst](https://dunst-project.org/)

### Install

- latest [release](https://github.com/doums/bato/releases/latest)
- AUR [package](https://aur.archlinux.org/packages/bato)

### Configuration

The binary looks for the config file `bato.toml` located in
`$XDG_CONFIG_HOME/bato/` (default to `$HOME/.config/bato/`).\
If the config file is not found, bato prints an error and exits.

All config options are detailed [here](https://github.com/doums/bato/blob/master/bato.toml).

Example:

```toml
tick_rate = 2
critical_level = 5
low_level = 20
full_design = true

[charging]
summary = "Battery"
body = "Charging"
icon = "battery-good-charging"

[discharging]
summary = "Battery"
body = "Discharging"
icon = "battery-good"

[full]
summary = "Battery"
body = "Full"
icon = "battery-full"

[low]
summary = "Battery"
body = "Low"
icon = "battery-low"

[critical]
summary = "Battery"
body = "Critical!"
icon = "battery-caution"
urgency = "critical"
```

### Usage

Run bato via your window manager or desktop environment autostart system

Example with XMonad

```haskell
myStartupHook = do
    spawnOnce "dunst"
    spawnOnce "bato -lfile"
```

> [!TIP]
> By default bato logs to stdout. To log into a file
run with `-lfile`. Logs are located in `~/.cache/bato/`

```shell
bato -h
```

### License

Mozilla Public License 2.0
