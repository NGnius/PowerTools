import {init_usdpl, target_usdpl, init_embedded, call_backend} from "usdpl-front";

const USDPL_PORT: number = 44443;

// Utility

export function resolve(promise: Promise<any>, setter: any) {
    (async function () {
        let data = await promise;
        if (data != null) {
            console.debug("Got resolved", data);
            setter(data);
        } else {
            console.warn("Resolve failed:", data);
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

// CPU

export async function getCpuCount(): Promise<number> {
    return (await call_backend("CPU_count", []))[0];
}

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

// Memory

export async function getMemoryTransparentHugepagesEnabled(): Promise<'always' | 'madvise' | 'never'> {
    return (await call_backend("MEMORY_get_transparent_hugepages_enabled", []))[0];
}

export async function setMemoryTransparentHugepagesEnabled(state: 'always' | 'madvise' | 'never'): Promise<'always' | 'madvise' | 'never'> {
    return (await call_backend("MEMORY_set_transparent_hugepages_enabled", [state]))[0];
}

export async function unsetMemoryTransparentHugepagesEnabled(): Promise<'always' | 'madvise' | 'never'> {
    return (await call_backend("MEMORY_unset_transparent_hugepages_enabled", []))[0];
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

export async function getGeneralSettingsName(): Promise<boolean> {
    return (await call_backend("GENERAL_get_name", []))[0];
}

export async function waitForComplete(): Promise<boolean> {
    return (await call_backend("GENERAL_wait_for_unlocks", []))[0];
}
