# PowerTools

![plugin_demo](https://raw.githubusercontent.com/NGnius/PowerTools/master/extras/ui.png)

Steam Deck power tweaks for power users.

This is generated from the template plugin for the [SteamOS Plugin Loader](https://github.com/SteamDeckHomebrew/PluginLoader).

## Cool, whatever

Yeah, that's fair.
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

### Set Fan speed

Enable automatic control: `echo 0 > /sys/class/hwmon/hwmon5/recalculate` enables automatic fan control.

Disable automatic control: `echo 1 > /sys/class/hwmon/hwmon5/recalculate` disables automatic (temperature-based) fan control and starts using the set fan target instead.

Set the fan speed: `echo {rpm} > /sys/class/hwmon/hwmon5/fan1_target` where `{rpm}` is the RPM.

Read the actual fan RPM: `cat /sys/class/hwmon/hwmon5/fan1_input` gives the fan speed.

NOTE: There's a bug in the fan controller; if you enable automatic fan control it will forget any previously-set target despite it appearing to be set correctly (i.e. `cat /sys/class/hwmon/hwmon5/fan1_target` will display the correct value).
When you disable automatic fan control, you will need to set the fan RPM again.

### Steam Deck kernel patches

This is how I figured out how the fan stuff works.
I've only scratched the surface of what this code allows, I'm sure it has more useful information.
https://lkml.org/lkml/2022/2/5/391

## License

This is licensed under GNU GPLv3.
