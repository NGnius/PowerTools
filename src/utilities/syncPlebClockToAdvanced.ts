import { Cpu, General, getValue, setValue } from "../usdplFront";

export function syncPlebClockToAdvanced() {
    const cpuCount = getValue(General.LimitsAll).cpu.count;
    const minClock = getValue(Cpu.MinClock);
    const maxClock = getValue(Cpu.MaxClock);
    const clockArr = [];
    for (let i = 0; i < cpuCount; i++) {
        clockArr.push({
            min: minClock,
            max: maxClock,
        });
    }
    setValue(Cpu.MinmaxClocks, clockArr);
}
