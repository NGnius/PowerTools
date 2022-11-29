import { CPU_BE } from "../usdplBackend";
import { CPU, setValue } from "../usdpl";
import { SETTINGS_LIMITS } from "./settingsLimits";

export function syncPlebClockToAdvanced() {
    const cpuCount = SETTINGS_LIMITS.cpu.count;
    const minClock = CPU_BE[CPU.MinClock];
    const maxClock = CPU_BE[CPU.MaxClock];
    const clockArr = [];
    for (let i = 0; i < cpuCount; i++) {
        clockArr.push({
            min: minClock,
            max: maxClock,
        });
    }
    setValue(CPU.MinmaxClocks, clockArr);
}
