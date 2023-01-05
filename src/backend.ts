import {init_usdpl, target_usdpl, init_embedded, call_backend} from "usdpl-front";

const USDPL_PORT: number = 44443;

// Utility

export function resolve<T>(promise: Promise<T>, setter: (t: T) => void) {
    (async function () {
        let data = await promise;
        if (data != null) {
            console.debug("Got resolved", data);
            setter(data);
        } else {
            console.warn("Resolve failed:", data);
            log(LogLevel.Warn, "");
        }
    })();
}

export async function initBackend() {
    // init usdpl
    await init_embedded();
    init_usdpl(USDPL_PORT);
    console.log("USDPL started for framework: " + target_usdpl());
    //setReady(true);
}

// API limit types

export type RangeLimit = {
    min: number;
    max: number;
};

export type SettingsLimits = {
    battery: BatteryLimits;
    cpu: CpusLimits;
    gpu: GpuLimits;
    general: GeneralLimits;
};

export type BatteryLimits = {
    charge_current: RangeLimit | null;
    charge_current_step: number;
    charge_modes: string[];
};

export type CpuLimits = {
    clock_min_limits: RangeLimit | null;
    clock_max_limits: RangeLimit | null;
    clock_step: number;
    governors: string[];
};

export type CpusLimits = {
    cpus: CpuLimits[];
    count: number;
    smt_capable: boolean;
};

export type GeneralLimits = {};

export type GpuLimits = {
    fast_ppt_limits: RangeLimit | null;
    slow_ppt_limits: RangeLimit | null;
    ppt_step: number;
    clock_min_limits: RangeLimit | null;
    clock_max_limits: RangeLimit | null;
    clock_step: number;
    memory_control_capable: boolean;
};

// API

export async function getInfo(): Promise<string> {
    return (await call_backend("V_INFO", []))[0];
}

// Battery

export async function getBatteryCurrent(): Promise<number> {
    return (await call_backend("BATTERY_current_now", []))[0];
}

export async function getBatteryChargeNow(): Promise<number> {
    return (await call_backend("BATTERY_charge_now", []))[0];
}

export async function getBatteryChargeFull(): Promise<number> {
    return (await call_backend("BATTERY_charge_full", []))[0];
}

export async function getBatteryChargeDesign(): Promise<number> {
    return (await call_backend("BATTERY_charge_design", []))[0];
}

export async function getBatteryChargeRate(): Promise<number> {
    return (await call_backend("BATTERY_get_charge_rate", []))[0];
}

export async function setBatteryChargeRate(val: number): Promise<number> {
    return (await call_backend("BATTERY_set_charge_rate", [val]))[0];
}

export async function unsetBatteryChargeRate(): Promise<any[]> {
    return await call_backend("BATTERY_unset_charge_rate", []);
}

export async function getBatteryChargeMode(): Promise<string> {
    return (await call_backend("BATTERY_get_charge_mode", []))[0];
}

export async function setBatteryChargeMode(val: string): Promise<string> {
    return (await call_backend("BATTERY_set_charge_mode", [val]))[0];
}

export async function unsetBatteryChargeMode(): Promise<any[]> {
    return await call_backend("BATTERY_unset_charge_mode", []);
}

// CPU

export async function setCpuSmt(status: boolean): Promise<boolean> {
    return (await call_backend("CPU_set_smt", [status]))[0];
}

/*export async function getCpuCount(): Promise<number> {
    return (await call_backend("CPU_count", []))[0];
}*/

export async function setCpuOnline(index: number, online: boolean): Promise<boolean> {
    return (await call_backend("CPU_set_online", [index, online]))[0];
}

export async function setCpuOnlines(onlines: boolean[]): Promise<boolean[]> {
    return await call_backend("CPU_set_onlines", onlines);
}

export async function getCpusOnline(): Promise<boolean[]> {
    return (await call_backend("CPU_get_onlines", [])); // -> online status for all CPUs
}

export async function setCpuClockLimits(index: number, min: number, max: number): Promise<number[]> {
    return (await call_backend("CPU_set_clock_limits", [index, min, max])); // -> [min, max]
}

export async function getCpuClockLimits(index: number): Promise<number[]> {
    return (await call_backend("CPU_get_clock_limits", [index])); // -> [min, max]
}

export async function unsetCpuClockLimits(index: number): Promise<any[]> {
    return (await call_backend("CPU_unset_clock_limits", [index]));
}

export async function setCpuGovernor(index: number, val: string): Promise<string> {
    return (await call_backend("CPU_set_governor", [index, val]))[0];
}

export async function getCpusGovernor(): Promise<string[]> {
    return (await call_backend("CPU_get_governors", [])); // -> governors for all CPUs
}

// GPU

export async function setGpuPpt(fast: number, slow: number): Promise<number[]> {
    return (await call_backend("GPU_set_ppt", [fast, slow])); // -> [fastPPT, slowPPT]
}

export async function getGpuPpt(): Promise<number[]> {
    return (await call_backend("GPU_get_ppt", [])); // -> [fastPPT, slowPPT]
}

export async function unsetGpuPpt(): Promise<any[]> {
    return (await call_backend("GPU_unset_ppt", []));
}

export async function setGpuClockLimits(min: number, max: number): Promise<number[]> {
    return (await call_backend("GPU_set_clock_limits", [min, max])); // -> [min, max]
}

export async function getGpuClockLimits(): Promise<number[]> {
    return (await call_backend("GPU_get_clock_limits", [])); // -> [min, max]
}

export async function unsetGpuClockLimits(): Promise<any[]> {
    return (await call_backend("GPU_unset_clock_limits", []));
}

export async function setGpuSlowMemory(val: boolean): Promise<boolean> {
    return (await call_backend("GPU_set_slow_memory", [val]))[0];
}

export async function getGpuSlowMemory(): Promise<boolean> {
    return (await call_backend("GPU_get_slow_memory", []))[0];
}

// general

export async function setGeneralPersistent(val: boolean): Promise<boolean> {
    return (await call_backend("GENERAL_set_persistent", [val]))[0];
}

export async function getGeneralPersistent(): Promise<boolean> {
    return (await call_backend("GENERAL_get_persistent", []))[0];
}

export async function loadGeneralSettings(path: string, name: string): Promise<boolean> {
    return (await call_backend("GENERAL_load_settings", [path, name]))[0];
}

export async function loadGeneralDefaultSettings(): Promise<boolean> {
    return (await call_backend("GENERAL_load_default_settings", []))[0];
}

export async function loadGeneralSystemSettings(): Promise<boolean> {
    return (await call_backend("GENERAL_load_system_settings", []))[0];
}

export async function getGeneralSettingsName(): Promise<string> {
    return (await call_backend("GENERAL_get_name", []))[0];
}

export async function waitForComplete(): Promise<boolean> {
    return (await call_backend("GENERAL_wait_for_unlocks", []))[0];
}

export async function getLimits(): Promise<SettingsLimits> {
    return (await call_backend("GENERAL_get_limits", []))[0];
}

export async function getDriverProviderName(name: string): Promise<string> {
    return (await call_backend("GENERAL_get_provider", [name]))[0];
}

export enum LogLevel {
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
}

export async function log(level: LogLevel, msg: string): Promise<boolean> {
    return (await call_backend("LOG", [level, msg]))[0];
}

export async function idk(): Promise<boolean> {
    return (await call_backend("GENERAL_idk", []))[0];
}
