import {init_usdpl, target_usdpl, init_embedded, call_backend, init_tr} from "usdpl-front";

const USDPL_PORT: number = 44443;

// Utility

export function resolve<T>(promise: Promise<T>, setter: (t: T) => void) {
    (async function () {
        let data = await promise;
        if (data != null) {
            console.debug("Got resolved", data);
            setter(data);
        } else {
            console.warn("Resolve failed:", data, promise);
            log(LogLevel.Warn, "A resolve failed");
        }
    })();
}

export function resolve_nullable<T>(promise: Promise<T | null>, setter: (t: T | null) => void) {
    (async function () {
        let data = await promise;
        console.debug("Got resolved", data);
        setter(data);
    })();
}

export async function initBackend() {
    // init usdpl
    await init_embedded();
    init_usdpl(USDPL_PORT);
    console.log("POWERTOOLS: USDPL started for framework: " + target_usdpl());
    const user_locale =
        navigator.languages && navigator.languages.length
            ? navigator.languages[0]
            : navigator.language;
    console.log("POWERTOOLS: locale", user_locale);
    //let mo_path = "../plugins/PowerTools/translations/" + user_locale.toString() + ".mo";
    await init_tr(user_locale);
    //await init_tr("../plugins/PowerTools/translations/test.mo");
    //setReady(true);
}

export type IdcProps = {
    idc: any;
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
    charge_limit: RangeLimit | null;
    charge_limit_step: number;
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
    governors: string[];
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

export async function getBatteryChargePower(): Promise<number> {
    return (await call_backend("BATTERY_charge_power", []))[0];
}

export async function getBatteryChargeRate(): Promise<number | null> {
    return (await call_backend("BATTERY_get_charge_rate", []))[0];
}

export async function setBatteryChargeRate(val: number): Promise<number> {
    return (await call_backend("BATTERY_set_charge_rate", [val]))[0];
}

export async function unsetBatteryChargeRate(): Promise<any[]> {
    return await call_backend("BATTERY_unset_charge_rate", []);
}

export async function getBatteryChargeMode(): Promise<string | null> {
    return (await call_backend("BATTERY_get_charge_mode", []))[0];
}

export async function setBatteryChargeMode(val: string): Promise<string> {
    return (await call_backend("BATTERY_set_charge_mode", [val]))[0];
}

export async function unsetBatteryChargeMode(): Promise<any[]> {
    return await call_backend("BATTERY_unset_charge_mode", []);
}

export async function getBatteryChargeLimit(): Promise<number | null> {
    return (await call_backend("BATTERY_get_charge_limit", []))[0];
}

export async function setBatteryChargeLimit(val: number): Promise<number> {
    return (await call_backend("BATTERY_set_charge_limit", [val]))[0];
}

export async function unsetBatteryChargeLimit(): Promise<any[]> {
    return await call_backend("BATTERY_unset_charge_limit", []);
}

// CPU

export async function setCpuSmt(status: boolean): Promise<boolean[]> {
    return await call_backend("CPU_set_smt", [status]);
}

export async function getCpuSmt(): Promise<boolean> {
    return (await call_backend("CPU_get_smt", []))[0];
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

export async function loadGeneralSettings(id: string, name: string): Promise<boolean> {
    return (await call_backend("GENERAL_load_settings", [id, name]))[0];
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

export async function getGeneralSettingsPath(): Promise<string> {
    return (await call_backend("GENERAL_get_path", []))[0];
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

export async function forceApplySettings(): Promise<boolean> {
    return (await call_backend("GENERAL_apply_now", []))[0];
}

export async function onPluggedIn(): Promise<boolean> {
    return (await call_backend("GENERAL_on_pluggedin", []))[0];
}

export async function onUnplugged(): Promise<boolean> {
    return (await call_backend("GENERAL_on_unplugged", []))[0];
}

export type Message = {
    /// Message identifier
    id: number | null,
    /// Message title
    title: string,
    /// Message content
    body: string,
    /// Link for further information
    url: string | null,
};

export async function getMessages(since: number | null): Promise<Message[]> {
    return (await call_backend("MESSAGE_get", [since]));
}

export async function dismissMessage(id: number): Promise<boolean> {
    return (await call_backend("MESSAGE_dismiss", [id]))[0];
}

export type Periodicals = {
    battery_current: number | null,
    battery_charge_now: number | null,
    battery_charge_full: number | null,
    battery_charge_power: number | null,
    settings_path: string | null,
};

export async function getPeriodicals(): Promise<Periodicals> {
    const result: any[] = await call_backend("GENERAL_get_periodicals", []);
    return {
        battery_current: result[0],
        battery_charge_now: result[1],
        battery_charge_full: result[2],
        battery_charge_power: result[3],
        settings_path: result[4],
    };
}
