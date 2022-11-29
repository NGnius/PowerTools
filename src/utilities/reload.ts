import { GENERAL_BE } from "../usdplBackend";
import { BACKEND_CALLS, callBackend, setValue, BATTERY, CPU, GPU, GENERAL } from "../usdpl";
import { countCpus } from "./helpers";
import { syncPlebClockToAdvanced } from "./syncPlebClockToAdvanced";

const getPeriodicCalls = () => [
    callBackend(BACKEND_CALLS.BatteryCurrentNow, []).then(([data]) => setValue(BATTERY.CurrentNow, data)),
    callBackend(BACKEND_CALLS.BatteryChargeNow, []).then(([data]) => setValue(BATTERY.ChargeNow, data)),
    callBackend(BACKEND_CALLS.BatteryChargeFull, []).then(([data]) => setValue(BATTERY.ChargeFull, data)),
    callBackend(BACKEND_CALLS.GeneralGetPersistent, []).then(([ok]) => setValue(GENERAL.Persistent, ok)),
];

const getAllCalls = (smtAllowed: boolean) => [
    ...getPeriodicCalls(),
    callBackend(BACKEND_CALLS.GeneralGetLimits, []).then(([limits]) => {
        setValue(GENERAL.LimitsAll, limits);
        console.debug("POWERTOOLS: got limits", limits);
    }),
    callBackend(BACKEND_CALLS.BatteryGetChargeRate, []).then(([rate]) => setValue(BATTERY.ChargeRate, rate)),
    callBackend(BACKEND_CALLS.BatteryChargeDesign, []).then(([rate]) => setValue(BATTERY.ChargeDesign, rate)),
    callBackend(BACKEND_CALLS.CpuGetOnlines, []).then((statii) => {
        setValue(CPU.StatusOnline, statii);
        setValue(CPU.Online, countCpus(statii));
        setValue(CPU.Smt, statii.length > 3 && statii[0] === statii[1] && statii[2] === statii[3]);
        setValue(CPU.Smt, statii.length > 3 && statii[0] === statii[1] && statii[2] === statii[3] && smtAllowed);
    }),
    callBackend(BACKEND_CALLS.CpuGetClockLimits, [0]).then((limits) => {
        setValue(CPU.MaxClock, limits[0]);
        setValue(CPU.MaxClock, limits[1]);
        syncPlebClockToAdvanced();
    }),
    callBackend(BACKEND_CALLS.CpuGetGovernors, []).then((governors) => setValue(CPU.Governor, governors)),
    callBackend(BACKEND_CALLS.GpuGetPpt, []).then((ppts) => {
        setValue(GPU.FastPpt, ppts[0]);
        setValue(GPU.SlowPpt, ppts[1]);
    }),
    callBackend(BACKEND_CALLS.GpuGetClockLimits, []).then((limits) => {
        setValue(GPU.MinClock, limits[0]);
        setValue(GPU.MaxClock, limits[1]);
    }),
    callBackend(BACKEND_CALLS.GpuGetSlowMemory, []).then(([status]) => setValue(GPU.SlowMemory, status)),
    callBackend(BACKEND_CALLS.GeneralGetName, []).then(([name]) => setValue(GENERAL.Name, name)),
    callBackend(BACKEND_CALLS.VInfo, []).then(([info]) => setValue(GENERAL.VInfo, info)),
];

/** this function reloads GENERAL_name and determines if all backends should reload */
const getShouldReloadAll = () =>
    callBackend(BACKEND_CALLS.GeneralGetName, []).then(([newName]) => {
        const prevName = GENERAL_BE[GENERAL.Name];
        setValue(GENERAL.Name, newName);
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
