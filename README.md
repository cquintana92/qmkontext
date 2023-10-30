# QMKontext

[![Lint](https://github.com/cquintana92/qmkontext/actions/workflows/lint.yaml/badge.svg)](https://github.com/cquintana92/qmkontext/actions/workflows/lint.yaml)
[![Release](https://github.com/cquintana92/qmkontext/actions/workflows/release.yaml/badge.svg)](https://github.com/cquintana92/qmkontext/actions/workflows/release.yaml)

QMKontext is a program that allows you to send your current computer context to your QMK keyboard so it can react to it.

As of now it only supports Linux x86_64, and the current program detector only works on X11 sessions, but in the future I'll try to add support for more configurations.

## How to get

You can download the latest deb file from [the releases page](https://github.com/cquintana92/qmkontext/releases/latest).

Then, install it by:

```
$ sudo apt install ./qmkontext-VERSION.deb
```

### Build from source

If you want, you can build it from source running:

```
$ cargo build --release
$ sudo mv target/release/qmkontext-cli /usr/bin/qmkontext-cli
```

The systemd service definition can be found [in the pkg directory](./pkg/qmkontext.service).

## QMKontext usage

When you install the `.deb` package, it will place a sample config file in `/etc/qmkontext/config.toml` which will contain all the possible options documented.

> NOTE: In order for the current program detection to work (only tested on Ubuntu 22.04 using X11 sessions), you'll need `xdotool` installed (which should have been installed as a dependency of the `.deb` package).

The first thing you will need to do is to fill your `[keyboard]` section of the config file. In order to help you with that, you can run `qmkontext list`, and that will list your available devices, with the `vendor_id` and `product_id`.
It will also print the `usage` and `usage_page`, which you will need in case they differ from the defaults.

QMKontext allows you, by default, to detect the currently focused program by configuring the `[[current_program.mappings]]` array, by setting the `key` to a string that can be found on either the program binary or the window name, and the `value` to whatever value you want to send to QMK.

It also allows you to run arbitrary commands (aka: custom bash scripts or one-liners) and send the result to QMK in the same fashion. You can add as many as you want as seen in the `[[custom_commands]]` array. The `command` can either be a `bash` one-line command or a path to a bash script. The output of the command/script must be a single number between 0 and 255, as it will be sent as the payload to the QMK keyboard.

For testing the config file without starting it in background, you can just run `qmkontext`. If you want to debug what it's detecting, feel free to change the `log_level` property on the config file. You can also pass `--config PATH_TO_FILE` if you want to test a different config file.

Once you are comfortable with the config file, make sure to place it in `/etc/qmkontext/config.toml` and configure the systemd service.

In order to do so:

```
$ sudo systemctl daemon-reload
$ sudo systemctl start qmkontext.service
```

If you want to see the logs, you can do it with the following command:

```
$ sudo journalctl -fu qmkontext.service
```

If you want the service to be started automatically with your system, you can run:

```
$ sudo systemctl enable qmkontext.service
```

## How does it work

QMKontext works by sending regular commands to your QMK keyboard by making use of the [QMK Raw HID](https://docs.qmk.fm/#/feature_rawhid) API.

It will send the commands using the following structure:

* `data[0]`: command id. This is used to identify which command is being sent.
* `data[1]`: command data. This contains the payload of the command.

A simple example is sending the current program. The buffer sent to the keyboard will contain:

* `data[0]`: the value set in the config file for the `current_program.command_id` variable.
* `data[1]`: the value of the corresponding program, if is the active one, or otherwise the `current_program.default_value` set in the config file.

## QMK setup

In order for this to work on your QMK keyboard, make sure to add the following to your `rules.mk` file:

```
RAW_ENABLE = yes
```

After that, you can add a section like this in your `keymap.c` file:

```c
// HID
#define COMMAND_CURRENT_PROGRAM 1
#define CURRENT_PROGRAM_DEFAULT 0
#define CURRENT_PROGRAM_CHROME 1
#define CURRENT_PROGRAM_FIREFOX 2
#define CURRENT_PROGRAM_PYCHARM 3

uint8_t current_program = CURRENT_PROGRAM_DEFAULT;

void raw_hid_receive(uint8_t *data, uint8_t length) {
    uint8_t command = data[0];
    uint8_t payload = data[1];
    switch (command) {
        case COMMAND_CURRENT_PROGRAM:
            current_program = payload;
            break;
    }
}
```

And later, in your QMK code, you can check the current program by checking the `current_program` variable.

## Troubleshooting

In order to read the logs of the background service, you can use:

```
$ sudo journalctl -fu qmkontext.service
```

If you see something like "cannot create xdo session", you will need to edit the service definition. For doing so, run:

```
$ sudo vim /etc/systemd/system/qmkontext.service
```

And under the `[Service]` directive, add:

```
[Service]
User=YOUR_USERNAME_HERE
```

In case that doesn't fix it, try replacing the `DISPLAY` variable of the systemd service.

Also, in case you want to see what program it's detecting, set the `log_level` directive to `debug` and it will print the detections it's making. Also, make sure that you are matching the casing and the `use_loweracase` flag of your config. 