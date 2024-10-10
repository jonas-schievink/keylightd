<div align="center">

# `keylightd`

### Keyboard backlight daemon for Framework laptops

</div>

`keylightd` is a small system daemon for [Framework] laptops that listens to keyboard and touchpad input, and turns on the keyboard backlight while either is being used.

[Framework]: https://frame.work/

This is a fork of the original `keylightd` to make it compatible with systems that use [keyd] or [kanata].

[keyd]: https://github.com/rvaiya/keyd
[kanata]: https://github.com/jtroo/kanata

## Installation

To install from source, clone the repository and run:

```shell
$ cargo build --release
$ sudo cp target/release/keylightd /usr/local/bin
```

`keylightd` has no native dependencies you have to install first (apart from a recent Rust toolchain for building it, of course).
It implements communication with the Embedded Controller itself, and talks to the input devices using `evdev` ioctls directly.
It also does not have any hard dependencies on a desktop environment or display server.

If you want to configure `keylightd` as a systemd service that starts on boot, you can use the provided service file:

```shell
$ sudo cp etc/keylightd.service /etc/systemd/system
$ sudo systemctl enable --now keylightd
```

## Running

Note that `keylightd` needs to be run as root, since it accesses the Embedded Controller to control the keyboard backlight.

`keylightd` takes the following command-line arguments:

```
Usage: keylightd [--brightness <brightness>] [--timeout <timeout>] [--power]

keylightd - automatic keyboard backlight daemon for Framework laptops

Options:
  --brightness      brightness level when active (0-100) [default=30]
  --timeout         activity timeout in seconds [default=10]
  --power           also control the power LED in the fingerprint module
  --help            display usage information
```

If you're using the provided `keylightd.service` file, you can adjust the command line parameters there.

## Contributing

This project does not accept contributions. It is finished and does what I want of it.
