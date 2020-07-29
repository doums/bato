## bato

Small program to send **bat**tery n**o**tifications when the level is critical, low and full. Coded in Rust (and C).

### features

* configuration in YAML

### prerequisite

- a notification server, like [Dunst](https://dunst-project.org/)
- libnotify

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
```

### license
Mozilla Public License 2.0
