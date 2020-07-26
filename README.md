## bato

Small program to send a notification when battery is critical, low, full. Coded in Rust (and a little bit of C).

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
bat_name: BAT0
low_level: 30
low:
  summary: Battery low
full:
  summary: Battery full
```

### license
Mozilla Public License 2.0
