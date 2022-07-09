# PowerTools

![plugin_demo](./assets/ui.png)

Steam Deck power tweaks for power users.

This is generated from the template plugin for the [SteamOS Plugin Loader](https://github.com/SteamDeckHomebrew/PluginLoader).
You will need that installed for this plugin to work.

## What does it do?

- Enable & disable CPU threads & SMT
- Set CPU max frequency and toggle boost
- Set some GPU power parameters (fastPPT & slowPPT)
- Display supplementary battery info
- Keep settings between restarts (stored in `~/.config/powertools/<appid>.json`)

## Cool, but that's too much work

Fair enough.
In case you still want some of the functionality, without the nice GUI, here's some equivalent commands.
These should all be run as superuser, i.e. run `sudo su` and then run these commands in that.

### Enable & Disable CPU threads

Enable: `echo 1 > /sys/devices/system/cpu/cpu{cpu_number}/online` where `{cpu_number}` is a number from 1 to 7 (inclusive).

Disable: `echo 0 > /sys/devices/system/cpu/cpu{cpu_number}/online` where `{cpu_number}` is a number from 1 to 7 (inclusive).

NOTE: You cannot enable or disable cpu0, hence why there are only 7 in the range for 8 cpu threads.

### Enable & Disable CPU boost

Enable: `echo 1 > /sys/devices/system/cpu/cpufreq/boost` enables boost across all threads.

Disable: `echo 0 > /sys/devices/system/cpu/cpufreq/boost` disables boost across all threads.

### Set CPU frequency

Use `cpupower` (usage: `cpupower --help`).
This isn't strictly how PowerTools does it, but it's a multi-step process which can involve changing the CPU governor.
All that can be done automatically by `cpupower frequency-set --freq {frequency}` where `{frequency}` is `1.7G`, `2.4G` or `2.8G`.

### Set GPU Power

Set Slow Powerplay Table (PPT):`echo {microwatts} > /sys/class/hwmon/hwmon4/power1_cap` where `{microwatts}` is a wattage in millionths of a Watt. This doesn't seem to do a lot.

Set Fast Powerplay Table (PPT): `echo {microwatts} > /sys/class/hwmon/hwmon4/power2_cap` where `{microwatts}` is a wattage in millionths of a Watt.

Get the entry limits for those two commands with `cat /sys/class/hwmon/hwmon4/power{number}_cap_max` where `{number}` is `1` (slowPPT) or `2` (fastPPT).

### Set Fan speed

NOTE: PowerTools no longer supports this, since [Fantastic](https://github.com/NGnius/Fantastic) does it much better.

Enable automatic control: `echo 0 > /sys/class/hwmon/hwmon5/recalculate` enables automatic fan control.

Disable automatic control: `echo 1 > /sys/class/hwmon/hwmon5/recalculate` disables automatic (temperature-based) fan control and starts using the set fan target instead.

Set the fan speed: `echo {rpm} > /sys/class/hwmon/hwmon5/fan1_target` where `{rpm}` is the RPM.

Read the actual fan RPM: `cat /sys/class/hwmon/hwmon5/fan1_input` gives the fan speed.

NOTE: There's a bug in the fan controller; if you enable automatic fan control it will forget any previously-set target despite it appearing to be set correctly (i.e. `cat /sys/class/hwmon/hwmon5/fan1_target` will display the correct value).
When you disable automatic fan control, you will need to set the fan RPM again.

### Battery stats

Get the battery charge right now: `cat /sys/class/hwmon/hwmon2/device/charge_now` gives charge in uAh (uAh * 7.7/1000000 = charge in Wh).

Get the maximum battery capacity: `cat /sys/class/hwmon/hwmon2/device/charge_full` gives charge in uAh.

Get the design battery capacity: `cat /sys/class/hwmon/hwmon2/device/charge_full_design` gives charge in uAh.

Get whether the deck is plugged in: `cat /sys/class/hwmon/hwmon5/curr1_input` gives the charger current in mA.

### Steam Deck kernel patches

This is how I figured out how the fan stuff works.
I've only scratched the surface of what this code allows, I'm sure it has more useful information.
https://lkml.org/lkml/2022/2/5/391

### Game launch detection

The biggest limitation right now is it can't detect a game closing -- only opening -- and only after PowerTools is looked at at least once (per SteamOS restart).

From a plugin, this can be accomplished by running some front-end Javascript.

```javascript
await execute_in_tab("SP", false,
    `SteamClient.Apps.RegisterForGameActionStart((actionType, data) => {
        console.log("start game", appStore.GetAppOverviewByGameID(data));
    });`
);
```

In PowerTools, the callback (the part surrounded by `{` and `}`, containing `console.log(...)`) sends a message to a local HTTP server to notify the PowerTools back-end that a game has been launched.

If you go to `http://127.0.0.1:5030` on your Steam Deck with PowerTools >=0.6.0, you can see some info about the last game you launched.

## License

This is licensed under GNU GPLv3.
