import { MinMax } from "../types";
import { get_value, set_value } from "./augmentedUsdplFront";

export function syncPlebClockToAdvanced() {
    const cpuCount = get_value("LIMITS_all").cpu.count;
    const minClock = get_value("CPUs_min_clock");
    const maxClock = get_value("CPUs_max_clock");
    const clockArr = [];
    for (let i = 0; i < cpuCount; i++) {
        clockArr.push({
            min: minClock,
            max: maxClock,
        } as MinMax);
    }
    set_value("CPUs_minmax_clocks", clockArr);
}
