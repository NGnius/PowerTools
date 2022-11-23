import { BackendCalls, callBackend, setValue, getValue, Battery, Cpu, Gpu, General } from "../usdplFront";
import { countCpus } from "./countCpus";
import { syncPlebClockToAdvanced } from "./syncPlebClockToAdvanced";

const getPeriodicCalls = () => [
    callBackend(BackendCalls.BatteryCurrentNow, []).then(([data]) => setValue(Battery.CurrentNow, data)),
    callBackend(BackendCalls.BatteryChargeNow, []).then(([data]) => setValue(Battery.ChargeNow, data)),
    callBackend(BackendCalls.BatteryChargeFull, []).then(([data]) => setValue(Battery.ChargeFull, data)),
    callBackend(BackendCalls.GeneralGetPersistent, []).then(([ok]) => setValue(General.Persistent, ok)),
];

const getAllCalls = (smtAllowed: boolean) => [
    ...getPeriodicCalls(),
    callBackend(BackendCalls.GeneralGetLimits, []).then(([limits]) => {
        setValue(General.LimitsAll, limits);
        console.debug("POWERTOOLS: got limits", limits);
    }),
    callBackend(BackendCalls.BatteryGetChargeRate, []).then(([rate]) => setValue(Battery.ChargeRate, rate)),
    callBackend(BackendCalls.BatteryChargeDesign, []).then(([rate]) => setValue(Battery.ChargeDesign, rate)),
    callBackend(BackendCalls.CpuGetOnlines, []).then((statii) => {
        setValue(Cpu.StatusOnline, statii);
        setValue(Cpu.Online, countCpus(statii));
        setValue(Cpu.Smt, statii.length > 3 && statii[0] === statii[1] && statii[2] === statii[3]);
        setValue(Cpu.Smt, statii.length > 3 && statii[0] === statii[1] && statii[2] === statii[3] && smtAllowed);
    }),
    callBackend(BackendCalls.CpuGetClockLimits, [0]).then((limits) => {
        setValue(Cpu.MaxClock, limits[0]);
        setValue(Cpu.MaxClock, limits[1]);
        syncPlebClockToAdvanced();
    }),
    callBackend(BackendCalls.CpuGetGovernors, []).then((governors) => setValue(Cpu.Governor, governors)),
    callBackend(BackendCalls.GpuGetPpt, []).then((ppts) => {
        setValue(Gpu.FastPpt, ppts[0]);
        setValue(Gpu.SlowPpt, ppts[1]);
    }),
    callBackend(BackendCalls.GpuGetClockLimits, []).then((limits) => {
        setValue(Gpu.MinClock, limits[0]);
        setValue(Gpu.MaxClock, limits[1]);
    }),
    callBackend(BackendCalls.GpuGetSlowMemory, []).then(([status]) => setValue(Gpu.SlowMemory, status)),
    callBackend(BackendCalls.GeneralGetName, []).then(([name]) => setValue(General.Name, name)),
    callBackend(BackendCalls.VInfo, []).then(([info]) => setValue(General.VInfo, info)),
];

/** this function reloads GENERAL_name and determines if all backends should reload */
const getShouldReloadAll = () =>
    callBackend(BackendCalls.GeneralGetName, []).then(([newName]) => {
        const prevName = getValue(General.Name);
        setValue(General.Name, newName);
        return newName !== prevName;
    });

export async function reload({
    usdplReady,
    fullReload,
    smtAllowed,
}: {
    usdplReady: boolean;
    fullReload: boolean;
    smtAllowed: boolean;
}): Promise<void> {
    if (!usdplReady) {
        return Promise.resolve();
    } else if ((await getShouldReloadAll()) || fullReload) {
        await Promise.all([...getPeriodicCalls(), ...getAllCalls(smtAllowed)]);
    } else {
        await Promise.all(getPeriodicCalls());
    }
}
