import time

class Plugin:
    CPU_COUNT = 8
    SCALING_FREQUENCIES = [1700000, 2400000, 2800000]
    
    # call from main_view.html with setCPUs(onclick_event) or call_plugin_method("set_cpus", count)
    async def set_cpus(self, count) -> int:
        print("Setting CPUs")
        with open("/home/deck/PowerTools.log", "a") as f:
            f.write(f"Setting {count} CPUs to active\n")
            f.flush()
            count = min(int(count), self.CPU_COUNT)
            # never touch cpu0, since it's special
            for cpu in range(1, count):
                f.write(f"Setting CPU {cpu} to online\n")
                enable_cpu(cpu)
            for cpu in range(count, self.CPU_COUNT):
                f.write(f"Setting CPU {cpu} to offline\n")
                disable_cpu(cpu)
        return self.CPU_COUNT
    
    async def get_cpus(self) -> int:
        online_count = 1 # cpu0 is always online
        for cpu in range(1, self.CPU_COUNT):
            online_count += int(status_cpu(cpu))
        return online_count
    
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
                path = cpu_freq_scaling_path(cpu)
                write_to_sys(path, selected_freq)
                updated += 1
        return updated

    async def get_max_boost(self) -> int:
        path = cpu_freq_scaling_path(0)
        freq = int(read_from_sys(path, amount=-1).strip())
        return self.SCALING_FREQUENCIES.index(freq)

    # Asyncio-compatible long-running code, executed in a task when the plugin is loaded
    async def _main(self):
        with open("/home/deck/PowerTools.log", "w") as f:
            f.write(f"Main loop\n")
        pass
            

# these are stateless (well, the state is not saved internally) functions, so there's no need for these to be called like a class method

def cpu_online_path(cpu_number: int) -> str:
    return f"/sys/devices/system/cpu/cpu{cpu_number}/online"

def cpu_freq_scaling_path(cpu_number: int) -> str:
    return f"/sys/devices/system/cpu/cpu{cpu_number}/cpufreq/scaling_setspeed"
    
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
