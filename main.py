import time
import os

VERSION = "0.4.2"

class CPU:
    SCALING_FREQUENCIES = [1700000, 2400000, 2800000]

    def __init__(self, number):
        self.number = number

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

    auto_fan = True
    
    async def get_version(self) -> str:
        return VERSION

    # CPU stuff
    
    # call from main_view.html with setCPUs(count, smt)
    async def set_cpus(self, count, smt=True):
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
        return online_count

    async def get_smt(self) -> bool:
        return self.smt
    
    async def set_boost(self, enabled: bool) -> bool:
        write_to_sys("/sys/devices/system/cpu/cpufreq/boost", int(enabled))
        return True
    
    async def get_boost(self) -> bool:
        return read_from_sys("/sys/devices/system/cpu/cpufreq/boost") == "1"

    async def set_max_boost(self, index):
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
        write_to_sys(gpu_power_path(power_number), value)
        return True

    async def get_gpu_power(self, power_number: int) -> int:
        return int(read_from_sys(gpu_power_path(power_number), amount=-1).strip())

    # Fan stuff

    async def set_fan_tick(self, tick: int):
        if tick >= len(self.FAN_SPEEDS):
            # automatic mode
            self.auto_fan = True
            write_to_sys("/sys/class/hwmon/hwmon5/recalculate", 0)
            write_to_sys("/sys/class/hwmon/hwmon5/fan1_target", 4099) # 4099 is default
            #subprocess.run(["systemctl", "start", "jupiter-fan-control.service"])
        else:
            # manual voltage
            self.auto_fan = False
            write_to_sys("/sys/class/hwmon/hwmon5/recalculate", 1)
            write_to_sys("/sys/class/hwmon/hwmon5/fan1_target", self.FAN_SPEEDS[tick])
            #subprocess.run(["systemctl", "stop", "jupiter-fan-control.service"])

    async def get_fan_tick(self) -> int:
        fan_target = int(read_from_sys("/sys/class/hwmon/hwmon5/fan1_target", amount=-1).strip())
        fan_input = int(read_from_sys("/sys/class/hwmon/hwmon5/fan1_input", amount=-1).strip())
        fan_target_v = float(fan_target) / 1000
        fan_input_v = float(fan_input) / 1000
        if self.auto_fan:
            return len(self.FAN_SPEEDS)
        elif fan_target == 4099 or (int(round(fan_target_v)) != int(round(fan_input_v)) and fan_target not in self.FAN_SPEEDS):
            # cannot read /sys/class/hwmon/hwmon5/recalculate, so guess based on available fan info
            # NOTE: the fan takes time to ramp up, so fan_target will never approximately equal fan_input
            # when fan_target was changed recently (hence set voltage caching)
            return len(self.FAN_SPEEDS)
        else:
            # quantize voltage to nearest tick (price is right rules; closest without going over)
            for i in range(len(self.FAN_SPEEDS)-1):
                if fan_target <= self.FAN_SPEEDS[i]:
                    return i
            return len(self.FAN_SPEEDS)-1 # any higher value is considered as highest manual setting

    async def fantastic_installed(self) -> bool:
        return os.path.exists("/home/deck/homebrew/plugins/Fantastic")

    # Battery stuff

    async def get_charge_now(self) -> int:
        return int(read_from_sys("/sys/class/hwmon/hwmon2/device/charge_now", amount=-1).strip())

    async def get_charge_full(self) -> int:
        return int(read_from_sys("/sys/class/hwmon/hwmon2/device/charge_full", amount=-1).strip())

    async def get_charge_design(self) -> int:
        return int(read_from_sys("/sys/class/hwmon/hwmon2/device/charge_full_design", amount=-1).strip())

    # Asyncio-compatible long-running code, executed in a task when the plugin is loaded
    async def _main(self):
        pass

    # called from main_view::onViewReady
    async def on_ready(self):
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


# these are stateless (well, the state is not saved internally) functions, so there's no need for these to be called like a class method

def cpu_online_path(cpu_number: int) -> str:
    return f"/sys/devices/system/cpu/cpu{cpu_number}/online"

def cpu_freq_scaling_path(cpu_number: int) -> str:
    return f"/sys/devices/system/cpu/cpu{cpu_number}/cpufreq/scaling_setspeed"

def cpu_governor_scaling_path(cpu_number: int) -> str:
    return f"/sys/devices/system/cpu/cpu{cpu_number}/cpufreq/scaling_governor"

def gpu_power_path(power_number: int) -> str:
    return f"/sys/class/hwmon/hwmon4/power{power_number}_cap"
    
def write_to_sys(path, value: int):
    with open(path, mode="w") as f:
        f.write(str(value))

def read_from_sys(path, amount=1):
    with open(path, mode="r") as f:
        return f.read(amount)
