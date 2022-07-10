import time
import os
import json
import asyncio
import pathlib
import subprocess

VERSION = "0.7.0"
HOME_DIR = "/home/deck"
DEFAULT_SETTINGS_LOCATION = HOME_DIR + "/.config/powertools/default_settings.json"
LOG_LOCATION = "/tmp/powertools.log"
FANTASTIC_INSTALL_DIR = HOME_DIR + "/homebrew/plugins/Fantastic"

import logging

logging.basicConfig(
    filename = LOG_LOCATION,
    format = '%(asctime)s %(levelname)s %(message)s',
    filemode = 'w',
    force = True)

logger = logging.getLogger()
logger.setLevel(logging.DEBUG)
logging.info(f"PowerTools v{VERSION} https://github.com/NGnius/PowerTools")
logging.debug(f"CWD: {os.getcwd()} HOME:{HOME_DIR}")

import sys
#import pathlib
sys.path.append(str(pathlib.Path(__file__).parent.resolve()))
import server as pt_server

startup_time = time.time()

class CPU:
    SCALING_FREQUENCIES = [1700000, 2400000, 2800000]

    def __init__(self, number, settings=None):
        self.number = number

        if settings is not None:
            self.set_max_boost(settings["max_boost"])
            if settings["online"]:
                self.enable()
            else:
                self.disable()
            # TODO governor

        if(self.status()):
            self.max_boost = self._get_max_boost()
        else:
            self.max_boost = CPU.SCALING_FREQUENCIES[-1]

    def enable(self):
        # CPU number 0 is special
        if(self.number == 0):
            return

        filepath = cpu_online_path(self.number)
        write_to_sys(filepath, 1)

        # The user might have changed the maximum cpu clock while the cpu was offline
        self._set_max_boost(self.max_boost)

    def disable(self):
        # CPU number 0 is special
        if(self.number == 0):
            return

        filepath = cpu_online_path(self.number)
        write_to_sys(filepath, 0)

    def set_max_boost(self, frequency):
        self.max_boost = frequency
        if(self.status()):
            self._set_max_boost(frequency)

    def status(self) -> bool:
        # cpu number 0 is always online
        if(self.number == 0):
            return True

        filepath = cpu_online_path(self.number)
        return read_from_sys(filepath) == "1"

    def governor(self) -> str:
        return self._read_scaling_governor()

    def settings(self) -> dict:
        return {
            "online": self.status(),
            "max_boost": self.max_boost,
            "governor": self.governor(),
        }

    def _read_scaling_governor(self) -> str:
        filepath = cpu_governor_scaling_path(self.number)
        return read_from_sys(filepath, amount=-1).strip()

    def _write_scaling_governor(self, governor: str):
        filepath = cpu_governor_scaling_path(self.number)
        with open(filepath, mode="w") as f:
            f.write(governor)

    def _set_max_boost(self, frequency):
        if(frequency == CPU.SCALING_FREQUENCIES[-1]):
            self._write_scaling_governor("schedutil")
            return

        if(self._read_scaling_governor() != "userspace"):
            self._write_scaling_governor("userspace")
        else:
            filepath = cpu_freq_scaling_path(self.number)
            write_to_sys(filepath, frequency)

    def _get_max_boost(self) -> int:
        filepath = cpu_freq_scaling_path(self.number)
        freq_maybe = read_from_sys(filepath, amount=-1).strip()

        if(freq_maybe is None or len(freq_maybe) == 0 or freq_maybe == "<unsupported>"):
            return CPU.SCALING_FREQUENCIES[-1]

        freq = int(freq_maybe)
        return freq
        

class Plugin:
    CPU_COUNT = 8
    FAN_SPEEDS = [0, 1000, 2000, 3000, 4000, 5000, 6000]

    gpu_power_values = [[-1, -1, -1], [1000000, 15000000, 29000000], [0, 15000000, 30000000]]

    auto_fan = True
    persistent = True
    modified_settings = False
    current_gameid = None
    old_gameid = None
    ready = False
    
    async def get_version(self) -> str:
        return VERSION

    # CPU stuff
    
    # call from main_view.html with setCPUs(count, smt)
    async def set_cpus(self, count, smt=True):
        logging.info(f"set_cpus({count}, {smt})")
        self.modified_settings = True
        cpu_count = len(self.cpus)
        self.smt = smt
        # print("Setting CPUs")
        if smt:
            count = min(int(count), cpu_count)
            for cpu in self.cpus[: count]:
                cpu.enable()
            for cpu in self.cpus[count :: 1]:
                cpu.disable()
        else:
            count = min(int(count), cpu_count / 2)
            # never touch cpu0, since it's special
            for cpu in self.cpus[1 : cpu_count : 2]:
                cpu.disable()
            for cpu in self.cpus[2 : cpu_count : 2]:
                if(cpu.number / 2 + 1 > count):
                    cpu.disable()
                else:
                    cpu.enable()

    async def get_cpus(self) -> int:
        online_count = 0
        for cpu in self.cpus:
            if(cpu.status()):
                online_count += 1
        logging.info(f"get_cpus() -> {online_count}")
        return online_count

    async def get_smt(self) -> bool:
        logging.info(f"get_smt() -> {self.smt}")
        return self.smt
    
    async def set_boost(self, enabled: bool) -> bool:
        self.modified_settings = True
        write_cpu_boost(enabled)
        return True
    
    async def get_boost(self) -> bool:
        return read_cpu_boost()

    async def set_max_boost(self, index):
        self.modified_settings = True
        if index < 0 or index >= len(CPU.SCALING_FREQUENCIES):
            return 0

        selected_freq = CPU.SCALING_FREQUENCIES[index]

        for cpu in self.cpus:
            cpu.set_max_boost(selected_freq)

        return len(self.cpus)

    async def get_max_boost(self) -> int:
        return CPU.SCALING_FREQUENCIES.index(self.cpus[0].max_boost)

    # GPU stuff

    async def set_gpu_power(self, value: int, power_number: int) -> bool:
        self.modified_settings = True
        write_gpu_ppt(power_number, value)
        return True

    async def get_gpu_power(self, power_number: int) -> int:
        return read_gpu_ppt(power_number)

    async def set_gpu_power_index(self, index: int, power_number: int) -> bool:
        if index < 3 and index >= 0:
            self.modified_settings = True
            old_value = read_gpu_ppt(power_number)
            if old_value not in self.gpu_power_values[power_number]:
                self.gpu_power_values[power_number][1] = old_value
            write_gpu_ppt(power_number, self.gpu_power_values[power_number][index])
            return True
        return False

    async def get_gpu_power_index(self, power_number: int) -> int:
        value = read_gpu_ppt(power_number)
        if value not in self.gpu_power_values[power_number]:
            #self.gpu_power_values[power_number][1] = value
            return 1
        else:
            return self.gpu_power_values[power_number].index(value)

    # Fan stuff

    async def set_fan_tick(self, tick: int):
        self.modified_settings = True
        if tick >= len(self.FAN_SPEEDS):
            # automatic mode
            self.enable_jupiter_fan_control(self)
            self.auto_fan = True
            write_to_sys("/sys/class/hwmon/hwmon5/recalculate", 0)
            write_to_sys("/sys/class/hwmon/hwmon5/fan1_target", 4099) # 4099 is default
        else:
            # manual voltage
            self.disable_jupiter_fan_control(self)
            self.auto_fan = False
            write_to_sys("/sys/class/hwmon/hwmon5/recalculate", 1)
            write_to_sys("/sys/class/hwmon/hwmon5/fan1_target", self.FAN_SPEEDS[tick])

    async def get_fan_tick(self) -> int:
        fan_target = read_fan_target()
        fan_input = int(read_from_sys("/sys/class/hwmon/hwmon5/fan1_input", amount=-1).strip())
        fan_target_v = float(fan_target) / 1000
        fan_input_v = float(fan_input) / 1000
        if self.auto_fan:
            return len(self.FAN_SPEEDS)
        elif fan_target == 4099 or (int(round(fan_target_v)) != int(round(fan_input_v)) and fan_target not in self.FAN_SPEEDS):
            # cannot read /sys/class/hwmon/hwmon5/recalculate, so guess based on available fan info
            # NOTE: the fan takes time to ramp up, so fan_target will never approximately equal fan_input
            # when fan_target was changed recently (hence set RPM caching)
            return len(self.FAN_SPEEDS)
        else:
            # quantize RPM to nearest tick (price is right rules; closest without going over)
            for i in range(len(self.FAN_SPEEDS)-1):
                if fan_target <= self.FAN_SPEEDS[i]:
                    return i
            return len(self.FAN_SPEEDS)-1 # any higher value is considered as highest manual setting

    async def fantastic_installed(self) -> bool:
        return os.path.exists(FANTASTIC_INSTALL_DIR)

    def disable_jupiter_fan_control(self):
        active = subprocess.Popen(["systemctl", "is-active", "jupiter-fan-control.service"]).wait() == 0
        if active:
            logging.info("Stopping jupiter-fan-control.service so it doesn't interfere")
            # only disable if currently active
            self.jupiter_fan_control_was_disabled = True
            stop_p = subprocess.Popen(["systemctl", "stop", "jupiter-fan-control.service"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            stop_p.wait()
            logging.debug("systemctl stop jupiter-fan-control.service stdout:\n" + stop_p.stdout.read().decode())
            logging.debug("systemctl stop jupiter-fan-control.service stderr:\n" + stop_p.stderr.read().decode())

    def enable_jupiter_fan_control(self):
        if self.jupiter_fan_control_was_disabled:
            logging.info("Starting jupiter-fan-control.service so it doesn't interfere")
            # only re-enable if I disabled it
            self.jupiter_fan_control_was_disabled = False
            start_p = subprocess.Popen(["systemctl", "start", "jupiter-fan-control.service"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            start_p.wait()
            logging.debug("systemctl start jupiter-fan-control.service stdout:\n" + start_p.stdout.read().decode())
            logging.debug("systemctl start jupiter-fan-control.service stderr:\n" + start_p.stderr.read().decode())

    # Battery stuff

    async def get_charge_now(self) -> int:
        return int(read_from_sys("/sys/class/hwmon/hwmon2/device/charge_now", amount=-1).strip())

    async def get_charge_full(self) -> int:
        return int(read_from_sys("/sys/class/hwmon/hwmon2/device/charge_full", amount=-1).strip())

    async def get_charge_design(self) -> int:
        return int(read_from_sys("/sys/class/hwmon/hwmon2/device/charge_full_design", amount=-1).strip())

    # Asyncio-compatible long-running code, executed in a task when the plugin is loaded
    async def _main(self):
        # startup: load & apply settings
        self.jupiter_fan_control_was_disabled = False
        if os.path.exists(DEFAULT_SETTINGS_LOCATION):
            settings = read_json(DEFAULT_SETTINGS_LOCATION)
            logging.debug(f"Loaded settings from {DEFAULT_SETTINGS_LOCATION}: {settings}")
        else:
            settings = None
            logging.debug(f"Settings {DEFAULT_SETTINGS_LOCATION} does not exist, skipped")
        if settings is None or settings["persistent"] == False:
            logging.debug("Ignoring settings from file")
            self.persistent = False
            self.guess_settings(self)
            self.modified_settings = True
        else:
            # apply settings
            logging.debug("Restoring settings from file")
            self.persistent = True
            self.apply_settings(self, settings)
            # self.modified_settings = False
        logging.info("Handled saved settings, back-end startup complete")
        # server setup
        await pt_server.start(VERSION)
        # work loop
        while True:
            # persistence
            if self.modified_settings and self.persistent:
                self.save_settings(self)
                self.modified_settings = False
            #self.reload_current_settings(self)

            await asyncio.sleep(1)
        await pt_server.shutdown()

    # called from main_view::onViewReady
    async def on_ready(self):
        delta = time.time() - startup_time
        if self.ready:
            logging.info(f"Front-end init called again {delta}s after startup")
            return
        logging.info(f"Front-end initialised {delta}s after startup")

    # persistence

    async def get_persistent(self) -> bool:
        return self.persistent

    async def set_persistent(self, enabled: bool):
        logging.debug(f"Persistence is now: {enabled}")
        self.persistent = enabled
        self.save_settings(self)

    def current_settings(self) -> dict:
        settings = dict()
        settings["cpu"] = self.current_cpu_settings(self)
        settings["gpu"] = self.current_gpu_settings(self)
        settings["fan"] = self.current_fan_settings(self)
        settings["persistent"] = self.persistent
        return settings

    def current_cpu_settings(self) -> dict:
        settings = dict()
        cpu_settings = []
        for cpu in self.cpus:
            cpu_settings.append(cpu.settings())
        settings["threads"] = cpu_settings
        settings["smt"] = self.smt
        settings["boost"] = read_cpu_boost()
        return settings

    def current_gpu_settings(self) -> dict:
        settings = dict()
        settings["slowppt"] = read_gpu_ppt(1)
        settings["fastppt"] = read_gpu_ppt(2)
        return settings

    def current_fan_settings(self) -> dict:
        settings = dict()
        settings["target"] = read_fan_target()
        settings["auto"] = self.auto_fan
        return settings

    def reload_current_settings(self):
        logging.debug(f"gameid update: {self.old_gameid} -> {self.current_gameid}")
        if self.persistent:
            # per-game profiles
            current_game = pt_server.http_server.game()
            self.old_gameid = self.current_gameid
            if current_game is not None and current_game.has_settings():
                self.current_gameid = current_game.gameid
                if self.old_gameid != self.current_gameid:
                    logging.info(f"Applying custom settings for {current_game.name()} {current_game.appid()}")
                    # new game; apply settings
                    settings = current_game.load_settings()
                    if settings is not None:
                        self.apply_settings(self, settings)
            else:
                self.current_gameid = None
                if self.old_gameid != None:
                    logging.info("Reapplying default settings; game without custom settings found")
                    self.old_gameid = None
                    # game without custom settings; apply defaults
                    settings = read_json(DEFAULT_SETTINGS_LOCATION)
                    self.apply_settings(self, settings)

    def save_settings(self):
        settings = self.current_settings(self)
        logging.debug(f"Saving settings to file: {settings}")
        current_game = pt_server.http_server.game()
        if current_game is not None and self.current_gameid is not None:
            save_location = current_game.settings_path()
        else:
            save_location = DEFAULT_SETTINGS_LOCATION
        write_json(save_location, settings)
        logging.info(f"Saved settings to {save_location}")

    def apply_settings(self, settings: dict):
        # CPU
        self.cpus = []

        for cpu_number in range(0, Plugin.CPU_COUNT):
            self.cpus.append(CPU(cpu_number, settings=settings["cpu"]["threads"][cpu_number]))
        self.smt = settings["cpu"]["smt"]
        write_cpu_boost(settings["cpu"]["boost"])
        # GPU
        write_gpu_ppt(1, settings["gpu"]["slowppt"])
        write_gpu_ppt(2, settings["gpu"]["fastppt"])
        # Fan
        if not (os.path.exists(FANTASTIC_INSTALL_DIR) or settings["fan"]["auto"]):
            self.disable_jupiter_fan_control(self)
            write_to_sys("/sys/class/hwmon/hwmon5/recalculate", 1)
            write_to_sys("/sys/class/hwmon/hwmon5/fan1_target", settings["fan"]["target"])
        elif settings["fan"]["auto"] and not os.path.exists(FANTASTIC_INSTALL_DIR):
            self.enable_jupiter_fan_control(self)


    def guess_settings(self):
        self.cpus = []
        for cpu_number in range(0, Plugin.CPU_COUNT):
            self.cpus.append(CPU(cpu_number))

        # If any core has two threads, smt is True
        self.smt = self.cpus[1].status()
        if(not self.smt):
            for cpu_number in range(2, len(self.cpus), 2):
                if(self.cpus[cpu_number].status()):
                    self.smt = True
                    break
        logging.info(f"SMT state is guessed to be {self.smt}")

    # per-game profiles

    async def get_current_game(self) -> str:
        current_game = pt_server.http_server.game()
        if current_game is None:
            return "Menu (default)"
        else:
            return f"{current_game.name()} ({current_game.appid()})"

    async def set_per_game_profile(self, enabled: bool):
        current_game = pt_server.http_server.game()
        if enabled and self.persistent and current_game is not None:
            self.current_gameid = current_game.gameid
            self.modified_settings = True
        else:
            if not enabled and current_game is not None and current_game.has_settings():
                # delete settings; disable settings loading
                os.remove(current_game.settings_path())
            self.current_gameid = None

    async def get_per_game_profile(self) -> bool:
        current_game = pt_server.http_server.game()
        return current_game is not None and current_game.has_settings()

    async def on_game_start(self, game_id: int, data) -> bool:
        pt_server.http_server.set_game(game_id, data)
        self.reload_current_settings(self)
        return True

    async def on_game_stop(self, game_id: int) -> bool:
        pt_server.http_server.unset_game(game_id)
        self.reload_current_settings(self)
        return True



# these are stateless (well, the state is not saved internally) functions, so there's no need for these to be called like a class method

def cpu_online_path(cpu_number: int) -> str:
    return f"/sys/devices/system/cpu/cpu{cpu_number}/online"

def cpu_freq_scaling_path(cpu_number: int) -> str:
    return f"/sys/devices/system/cpu/cpu{cpu_number}/cpufreq/scaling_setspeed"

def cpu_governor_scaling_path(cpu_number: int) -> str:
    return f"/sys/devices/system/cpu/cpu{cpu_number}/cpufreq/scaling_governor"

def gpu_power_path(power_number: int) -> str:
    return f"/sys/class/hwmon/hwmon4/power{power_number}_cap"

def read_cpu_boost() -> bool:
    return read_from_sys("/sys/devices/system/cpu/cpufreq/boost") == "1"

def write_cpu_boost(enable: bool):
    write_to_sys("/sys/devices/system/cpu/cpufreq/boost", int(enable))

def read_gpu_ppt(power_number: int) -> int:
    return read_sys_int(gpu_power_path(power_number))

def write_gpu_ppt(power_number:int, value: int):
    write_to_sys(gpu_power_path(power_number), value)

def read_fan_target() -> int:
    return read_sys_int("/sys/class/hwmon/hwmon5/fan1_target")
    
def write_to_sys(path, value: int):
    with open(path, mode="w") as f:
        f.write(str(value))
    logging.debug(f"Wrote `{value}` to {path}")

def read_from_sys(path, amount=1):
    with open(path, mode="r") as f:
        value = f.read(amount)
        logging.debug(f"Read `{value}` from {path}")
        return value

def read_sys_int(path) -> int:
    return int(read_from_sys(path, amount=-1).strip())

def write_json(path, data):
    with open(path, mode="w") as f:
        json.dump(data, f) # I always guess which is which param and I hate it

def read_json(path):
    with open(path, mode="r") as f:
        return json.load(f)

os_release = read_from_sys("/etc/os-release", amount=-1).strip()
logging.info(f"/etc/os-release\n{os_release}")
