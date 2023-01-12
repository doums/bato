[![bato](https://img.shields.io/github/actions/workflow/status/doums/bato/test.yml?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=github&style=for-the-badge)](https://github.com/doums/bato/actions?query=workflow%3Atest)
[![bato](https://img.shields.io/aur/version/bato?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=arch-linux&style=for-the-badge)](https://aur.archlinux.org/packages/bato/)

## bato

Small program to send **bat**tery n**o**tifications.

![bato](https://github.com/doums/bato/blob/master/img/bato.png)

- [features](#features)
- [prerequisite](#prerequisite)
- [install](#install)
- [AUR](#arch-linux-aur-package)
- [configuration](#configuration)
- [usage](#usage)
- [license](#license)

### Features

Configuration in YAML.

Notification events:

- level full
- level low
- level critical
- charging
- discharging

### Prerequisite

- a notification server, like [Dunst](https://dunst-project.org/)
- libnotify

### Install

- latest [release](https://github.com/doums/bato/releases/latest)
- AUR [package](https://aur.archlinux.org/packages/bato)

### Configuration

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

### Usage

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

Call this script from your window manager, _autostart_ programs.

### License

Mozilla Public License 2.0
