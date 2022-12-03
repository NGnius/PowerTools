import { CPU, getValue, setValue } from "../usdplFront";
import { SETTINGS_LIMITS } from "./settingsLimits";

export function syncPlebClockToAdvanced() {
    const cpuCount = SETTINGS_LIMITS.cpu.count;
    const minClock = getValue[CPU.MinClock];
    const maxClock = getValue[CPU.MaxClock];
    const clockArr = [];
    for (let i = 0; i < cpuCount; i++) {
        clockArr.push({
            min: minClock,
            max: maxClock,
        });
    }
    setValue(CPU.MinmaxClocks, clockArr);
}
