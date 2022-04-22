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

Disable: `echo 1 > /sys/devices/system/cpu/cpufreq/boost` disables boost across all threads.

### Set CPU frequency

Use `cpupower` (usage: `cpupower --help`).
This isn't strictly how PowerTools does it, but it's a multi-step process which can involve changing the CPU governor.
All that can be done automatically by `cpupower frequency-set --freq {frequency}` where `{frequency}` is `1.7G`, `2.4G` or `2.8G`.

## License

This is licensed under GNU GPLv3.
