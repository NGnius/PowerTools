import time
#import subprocess

VERSION = "0.3.0"

class Plugin:
    CPU_COUNT = 8
    SCALING_FREQUENCIES = [1700000, 2400000, 2800000]
    FAN_SPEEDS = [0, 1000, 2000, 3000, 4000, 5000, 6000]

    auto_fan = True
    
    async def get_version(self) -> str:
        return VERSION

    # CPU stuff
    
    # call from main_view.html with setCPUs(count, smt)
    async def set_cpus(self, count, smt=True) -> int:
        # print("Setting CPUs")
        if smt:
            count = min(int(count), self.CPU_COUNT)
            # never touch cpu0, since it's special
            for cpu in range(1, count):
                enable_cpu(cpu)
            for cpu in range(count, self.CPU_COUNT):
                disable_cpu(cpu)
            return self.CPU_COUNT
        else:
            count = min(int(count), self.CPU_COUNT / 2)
            for cpu in range(1, self.CPU_COUNT, 2):
                disable_cpu(cpu)
            for cpu in range(2, self.CPU_COUNT, 2):
                if (cpu / 2) + 1 > count:
                    disable_cpu(cpu)
                else:
                    enable_cpu(cpu)
    
    async def get_cpus(self) -> int:
        online_count = 1 # cpu0 is always online
        for cpu in range(1, self.CPU_COUNT):
            online_count += int(status_cpu(cpu))
        return online_count

    async def get_smt(self) -> bool:
        return status_cpu(1) == status_cpu(2) and status_cpu(3) == status_cpu(4)
    
    async def set_boost(self, enabled: bool) -> bool:
        write_to_sys("/sys/devices/system/cpu/cpufreq/boost", int(enabled))
        return True
    
    async def get_boost(self) -> bool:
        return read_from_sys("/sys/devices/system/cpu/cpufreq/boost") == "1"

    async def set_max_boost(self, index) -> int:
        if index >= len(self.SCALING_FREQUENCIES):
            return False
        selected_freq = self.SCALING_FREQUENCIES[index]
        updated = 0
        for cpu in range(0, self.CPU_COUNT):
            if cpu == 0 or status_cpu(cpu):
                if read_scaling_governor_cpu(cpu) != "userspace":
                    write_scaling_governor_cpu(cpu, "userspace")
                path = cpu_freq_scaling_path(cpu)
                if index == len(self.SCALING_FREQUENCIES) - 1:
                    write_scaling_governor_cpu(cpu, "schedutil")
                else:
                    write_to_sys(path, selected_freq)
                updated += 1
        return updated

    async def get_max_boost(self) -> int:
        path = cpu_freq_scaling_path(0)
        freq_maybe = read_from_sys(path, amount=-1).strip()
        if freq_maybe is None or len(freq_maybe) == 0 or freq_maybe == "<unsupported>":
            return len(self.SCALING_FREQUENCIES) - 1
        freq = int(freq_maybe)
        return self.SCALING_FREQUENCIES.index(freq)

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
            

# these are stateless (well, the state is not saved internally) functions, so there's no need for these to be called like a class method

def cpu_online_path(cpu_number: int) -> str:
    return f"/sys/devices/system/cpu/cpu{cpu_number}/online"

def cpu_freq_scaling_path(cpu_number: int) -> str:
    return f"/sys/devices/system/cpu/cpu{cpu_number}/cpufreq/scaling_setspeed"

def cpu_governor_scaling_path(cpu_number: int) -> str:
    return f"/sys/devices/system/cpu/cpu{cpu_number}/cpufreq/scaling_governor"
    
def write_to_sys(path, value: int):
    with open(path, mode="w") as f:
        f.write(str(value))

def read_from_sys(path, amount=1):
    with open(path, mode="r") as f:
        return f.read(amount)

def enable_cpu(cpu_number: int):
    filepath = cpu_online_path(cpu_number)
    write_to_sys(filepath, 1)

def disable_cpu(cpu_number: int):
    filepath = cpu_online_path(cpu_number)
    write_to_sys(filepath, 0)

def status_cpu(cpu_number: int) -> bool:
    filepath = cpu_online_path(cpu_number)
    return read_from_sys(filepath) == "1"

def read_scaling_governor_cpu(cpu_number: int) -> str:
    filepath = cpu_governor_scaling_path(cpu_number)
    return read_from_sys(filepath, amount=-1).strip()

def write_scaling_governor_cpu(cpu_number: int, governor: str):
    filepath = cpu_governor_scaling_path(cpu_number)
    with open(filepath, mode="w") as f:
        f.write(governor)
