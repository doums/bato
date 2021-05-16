[![bato](https://img.shields.io/github/workflow/status/doums/bato/Rust?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=github&style=for-the-badge)](https://github.com/doums/bato/actions?query=workflow%3ARust)
[![bato](https://img.shields.io/aur/version/bato?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=arch-linux&style=for-the-badge)](https://aur.archlinux.org/packages/bato/)

## bato

Small program to send **bat**tery n**o**tifications. Coded in Rust (and C).

![bato](https://github.com/doums/bato/blob/master/img/bato.png)

- [features](#features)
- [prerequisite](#prerequisite)
- [install](#install)
- [AUR](#arch-linux-aur-package)
- [configuration](#configuration)
- [usage](#usage)
- [license](#license)

### features

Configuration in YAML.

Notification events:
* level full
* level low
* level critical
* charging
* discharging

### prerequisite

- a notification server, like [Dunst](https://dunst-project.org/)
- libnotify

### install

Rust is a language that compiles to native code and by default statically links all dependencies.\
Simply download the latest [release](https://github.com/doums/bato/releases) of the compiled binary and use it! (do not forget to make it executable `chmod 755 bato`)

### Arch Linux AUR package

bato is present as a [package](https://aur.archlinux.org/packages/bato) in the Arch User Repository.

### configuration

The binary looks for the config file `bato.yaml` located in `$XDG_CONFIG_HOME/bato/` (default to `$HOME/.config/bato/`).\
If the config file is not found, bato prints an error and exits.\
All options are detailed [here](https://github.com/doums/bato/blob/master/bato.yaml).

Example:
```yaml
tick_rate: 1
critical_level: 5
low_level: 30
critical:
  summary: Critical battery level!
  body: Plug the power cable asap!
  icon: battery-caution
low:
  summary: Battery low
  icon: battery-040
full:
  summary: Battery full
  icon: battery-full
  urgency: Low
charging:
  summary: Battery
  body: Charging
  icon: battery-good-charging
discharging:
  summary: Battery
  body: Discharging
  icon: battery-good
```

### usage

Run bato as a _daemon_. For example launch it from a script
```
#!/usr/bin/bash

mapfile -t pids <<< "$(pgrep -x bato)"
if [ "${#pids[@]}" -gt 0 ]; then
  for pid in "${pids[@]}"; do
    if [ -n "$pid" ]; then
      kill "$pid"
    fi
  done
fi
bato &
```
and call this script from your windows manager, _autostart_ programs.

### license
Mozilla Public License 2.0
