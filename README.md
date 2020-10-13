## bato

Small program to send **bat**tery n**o**tifications. Coded in Rust (and C).

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

_Arch Linux AUR soon_

Rust is a language that compiles to native code and by default statically links all dependencies.\
Simply download the latest [release](https://github.com/doums/bato/releases) of the compiled binary and use it! (do not forget to make it executable `chmod 755 milcheck`)

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

### license
Mozilla Public License 2.0
