import {
    call_backend,
    get_value,
    set_value,
    init_embedded,
    init_usdpl,
    target_usdpl,
    version_usdpl,
} from "usdpl-front";
import { fill } from "./utilities/helpers";

export type MinMax = {
    min: number | null;
    max: number | null;
};

type RangeLimit = {
    min: number;
    max: number;
};

export type SettingsLimits = {
    battery: BatteryLimits;
    cpu: CpusLimits;
    gpu: GpuLimits;
    general: GeneralLimits;
};

type BatteryLimits = {
    charge_current: RangeLimit;
    charge_current_step: number;
    charge_modes: string[];
};

type CpuLimit = {
    clock_min_limits: RangeLimit;
    clock_max_limits: RangeLimit;
    clock_step: number;
    governors: string[];
};

type CpusLimits = {
    cpus: CpuLimit[];
    count: number;
    smt_capable: boolean;
};

type GeneralLimits = never;

type GpuLimits = {
    fast_ppt_limits: RangeLimit;
    slow_ppt_limits: RangeLimit;
    ppt_step: number;
    clock_min_limits: RangeLimit;
    clock_max_limits: RangeLimit;
    clock_step: number;
    memory_control_capable: boolean;
};

type ApiMessage<T extends unknown[] = never[]> = T;
type ApiCall<T extends unknown[], U extends unknown[]> = { params: ApiMessage<T>; return: ApiMessage<U> };

export enum BATTERY {
    ChargeDesign = "BATTERY_charge_design",
    ChargeFull = "BATTERY_charge_full",
    ChargeMode = "BATTERY_charge_mode",
    ChargeNow = "BATTERY_charge_now",
    ChargeRate = "BATTERY_charge_rate",
    CurrentNow = "BATTERY_current_now",
}

export enum CPU {
    Governor = "CPUs_governor",
    MaxClock = "CPUs_max_clock",
    MinClock = "CPUs_min_clock",
    MinmaxClocks = "CPUs_minmax_clocks",
    Online = "CPUs_online",
    Smt = "CPUs_SMT",
    StatusOnline = "CPUs_status_online",
}

export enum GPU {
    FastPpt = "GPU_fastPPT",
    MaxClock = "GPU_max_clock",
    MinClock = "GPU_min_clock",
    SlowMemory = "GPU_slow_memory",
    SlowPpt = "GPU_slowPPT",
}

export enum GENERAL {
    Name = "GENERAL_name",
    Persistent = "GENERAL_persistent",
    LimitsAll = "LIMITS_all",
    VInfo = "V_INFO",
}

export const enum BACKEND_CALLS {
    BatteryChargeDesign = "BATTERY_charge_design",
    BatteryChargeFull = "BATTERY_charge_full",
    BatteryChargeNow = "BATTERY_charge_now",
    BatteryCurrentNow = "BATTERY_current_now",
    BatteryGetChargeMode = "BATTERY_get_charge_mode",
    BatteryGetChargeRate = "BATTERY_get_charge_rate",
    BatterySetChargeMode = "BATTERY_set_charge_mode",
    BatterySetChargeRate = "BATTERY_set_charge_rate",
    BatteryUnsetChargeMode = "BATTERY_unset_charge_mode",
    BatteryUnsetChargeRate = "BATTERY_unset_charge_rate",
    CpuCount = "CPU_count",
    CpuGetClockLimits = "CPU_get_clock_limits",
    CpuGetGovernors = "CPU_get_governors",
    CpuGetOnlines = "CPU_get_onlines",
    CpuSetClockLimits = "CPU_set_clock_limits",
    CpuSetGovernor = "CPU_set_governor",
    CpuSetOnline = "CPU_set_online",
    CpuSetOnlines = "CPU_set_onlines",
    CpuSetSmt = "CPU_set_smt",
    CpuUnsetClockLimits = "CPU_unset_clock_limits",
    GeneralGetLimits = "GENERAL_get_limits",
    GeneralGetName = "GENERAL_get_name",
    GeneralGetPersistent = "GENERAL_get_persistent",
    GeneralIdk = "GENERAL_idk",
    GeneralLoadDefaultSettings = "GENERAL_load_default_settings",
    GeneralLoadSettings = "GENERAL_load_settings",
    GeneralLoadSystemSettings = "GENERAL_load_system_settings",
    GeneralSetPersistent = "GENERAL_set_persistent",
    GeneralWaitForUnlocks = "GENERAL_wait_for_unlocks",
    GpuGetClockLimits = "GPU_get_clock_limits",
    GpuGetPpt = "GPU_get_ppt",
    GpuGetSlowMemory = "GPU_get_slow_memory",
    GpuSetClockLimits = "GPU_set_clock_limits",
    GpuSetPpt = "GPU_set_ppt",
    GpuSetSlowMemory = "GPU_set_slow_memory",
    GpuUnsetClockLimits = "GPU_unset_clock_limits",
    GpuUnsetPpt = "GPU_unset_ppt",
    VInfo = "V_INFO",
}

type CallBackendTypes = {
    [BACKEND_CALLS.BatteryChargeDesign]: ApiCall<never[], [number]>;
    [BACKEND_CALLS.BatteryChargeFull]: ApiCall<never[], [number]>;
    [BACKEND_CALLS.BatteryChargeNow]: ApiCall<never[], [number]>;
    [BACKEND_CALLS.BatteryCurrentNow]: ApiCall<never[], [number]>;
    [BACKEND_CALLS.BatteryGetChargeMode]: ApiCall<never[], [string]>;
    [BACKEND_CALLS.BatteryGetChargeRate]: ApiCall<never[], [number]>;
    [BACKEND_CALLS.BatterySetChargeMode]: ApiCall<[string], [string]>;
    [BACKEND_CALLS.BatterySetChargeRate]: ApiCall<[rate: number], [number]>;
    [BACKEND_CALLS.BatteryUnsetChargeMode]: ApiCall<never[], unknown[]>;
    [BACKEND_CALLS.BatteryUnsetChargeRate]: ApiCall<never[], unknown[]>;
    [BACKEND_CALLS.CpuCount]: ApiCall<never[], [number]>;
    [BACKEND_CALLS.CpuGetClockLimits]: ApiCall<[index: number], [number, number]>;
    [BACKEND_CALLS.CpuGetGovernors]: ApiCall<never[], string[]>;
    [BACKEND_CALLS.CpuGetOnlines]: ApiCall<never[], boolean[]>;
    [BACKEND_CALLS.CpuSetClockLimits]: ApiCall<[index: number, min: number, max: number], [number, number]>;
    [BACKEND_CALLS.CpuSetGovernor]: ApiCall<[index: number, val: string], [string]>;
    [BACKEND_CALLS.CpuSetOnline]: ApiCall<[index: number, online: boolean], [boolean]>;
    [BACKEND_CALLS.CpuSetOnlines]: ApiCall<boolean[], boolean[]>;
    [BACKEND_CALLS.CpuSetSmt]: ApiCall<[status: boolean], [boolean]>;
    [BACKEND_CALLS.CpuUnsetClockLimits]: ApiCall<[index: number], unknown[]>;
    [BACKEND_CALLS.GeneralGetLimits]: ApiCall<never[], [SettingsLimits]>;
    [BACKEND_CALLS.GeneralGetName]: ApiCall<never[], [string]>;
    [BACKEND_CALLS.GeneralGetPersistent]: ApiCall<never[], [boolean]>;
    [BACKEND_CALLS.GeneralIdk]: ApiCall<unknown[], unknown[]>;
    [BACKEND_CALLS.GeneralLoadDefaultSettings]: ApiCall<never[], [boolean]>;
    [BACKEND_CALLS.GeneralLoadSettings]: ApiCall<[path: string, name: string], [boolean]>;
    [BACKEND_CALLS.GeneralLoadSystemSettings]: ApiCall<never[], boolean[]>;
    [BACKEND_CALLS.GeneralSetPersistent]: ApiCall<[boolean], [boolean]>;
    [BACKEND_CALLS.GeneralWaitForUnlocks]: ApiCall<never[], unknown[]>;
    [BACKEND_CALLS.GpuGetClockLimits]: ApiCall<never[], [number, number]>;
    [BACKEND_CALLS.GpuGetPpt]: ApiCall<never[], [number, number]>;
    [BACKEND_CALLS.GpuGetSlowMemory]: ApiCall<never[], [boolean]>;
    [BACKEND_CALLS.GpuSetClockLimits]: ApiCall<[min: number, max: number], [number, number]>;
    [BACKEND_CALLS.GpuSetPpt]: ApiCall<[fast: number, slow: number], [number, number]>;
    [BACKEND_CALLS.GpuSetSlowMemory]: ApiCall<[boolean], [boolean]>;
    [BACKEND_CALLS.GpuUnsetClockLimits]: ApiCall<never[], unknown[]>;
    [BACKEND_CALLS.GpuUnsetPpt]: ApiCall<never[], unknown[]>;
    [BACKEND_CALLS.VInfo]: ApiCall<never[], [string]>;
};

export type CpuTypes = {
    [CPU.Governor]: string[];
    [CPU.MaxClock]: number | null;
    [CPU.MinClock]: number | null;
    [CPU.MinmaxClocks]: MinMax[];
    [CPU.Online]: number;
    [CPU.Smt]: boolean;
    [CPU.StatusOnline]: boolean[];
};

export type BatteryTypes = {
    [BATTERY.ChargeDesign]: number;
    [BATTERY.ChargeFull]: number;
    [BATTERY.ChargeMode]: string | null;
    [BATTERY.ChargeNow]: number;
    [BATTERY.ChargeRate]: number | null;
    [BATTERY.CurrentNow]: number;
};

export type GpuTypes = {
    [GPU.FastPpt]: number | null;
    [GPU.MaxClock]: number | null;
    [GPU.MinClock]: number | null;
    [GPU.SlowMemory]: boolean;
    [GPU.SlowPpt]: number | null;
};

export type GeneralTypes = {
    [GENERAL.Name]: string;
    [GENERAL.Persistent]: boolean;
    [GENERAL.LimitsAll]: SettingsLimits;
    [GENERAL.VInfo]: string;
};

export type BackendTypes = CpuTypes & BatteryTypes & GpuTypes & GeneralTypes;

export const LIMITS: SettingsLimits = {
    cpu: {
        cpus: fill(
            {
                clock_max_limits: { min: 500, max: 3500 },
                clock_min_limits: { min: 1400, max: 3500 },
                clock_step: 100,
                governors: fill("").map((_, i) => i.toString()),
            },
            1
        ),
        count: 8,
        smt_capable: false,
    },
    gpu: {
        clock_max_limits: { min: 200, max: 1600 },
        clock_min_limits: { min: 200, max: 1600 },
        clock_step: 100,
        fast_ppt_limits: { min: 1000000, max: 29000000 },
        memory_control_capable: false,
        ppt_step: 1000000,
        slow_ppt_limits: { min: 1000000, max: 29000000 },
    },
    general: undefined as never,
    battery: {
        charge_current: { min: 250, max: 2500 },
        charge_current_step: 50,
        charge_modes: [],
    },
};

type SetValue = <T extends keyof BackendTypes>(key: T, value: BackendTypes[T]) => unknown;
type GetValue = <T extends keyof BackendTypes>(key: T) => BackendTypes[T];
type CallBackend = <K extends keyof CallBackendTypes>(
    name: K,
    parameters: CallBackendTypes[K]["params"]
) => Promise<CallBackendTypes[K]["return"]>;

let setValue = set_value as SetValue;
let getValue = get_value as GetValue;
let callBackend = call_backend as CallBackend;
let initEmbedded = init_embedded;
let initUsdpl = init_usdpl;
let targetUsdpl = target_usdpl;
let versionUsdpl = version_usdpl;

// locally mock the backend. useful for testing UI without affecting device
// hardware or if the backend isn't working. process.env.MOCK_BE is set in
// rollup config
if (process.env.MOCK_BE) {
    const defaults: BackendTypes = {
        [BATTERY.ChargeDesign]: 0,
        [BATTERY.ChargeFull]: 0,
        [BATTERY.ChargeMode]: null,
        [BATTERY.ChargeNow]: 0,
        [BATTERY.ChargeRate]: null,
        [BATTERY.CurrentNow]: 0,
        [CPU.Governor]: fill("").map((_, i) => i.toString()),
        [CPU.MaxClock]: null,
        [CPU.MinClock]: null,
        [CPU.MinmaxClocks]: fill({ min: null, max: null }),
        [CPU.Online]: 8,
        [CPU.Smt]: false,
        [CPU.StatusOnline]: [true, true, true, true, false, false, false, false],
        [GENERAL.LimitsAll]: LIMITS,
        [GENERAL.Name]: "NAME",
        [GENERAL.Persistent]: false,
        [GENERAL.VInfo]: "V_INFO",
        [GPU.FastPpt]: null,
        [GPU.MaxClock]: null,
        [GPU.MinClock]: null,
        [GPU.SlowMemory]: false,
        [GPU.SlowPpt]: null,
    };

    console.debug("using mock backend");
    const mockBackend = { ...defaults };
    callBackend = async () => new Array(8).fill(0); // callBackend returns string[], number[], or boolean[]. zero can be coerced to work in most of those situations?
    getValue = (key) => mockBackend[key];
    setValue = (key, value) => void (mockBackend[key] = value);
    initEmbedded = () => {};
    initUsdpl = () => {};
    targetUsdpl = () => "MOCK_BE";
    versionUsdpl = () => "MOCK_BE";
}
export { callBackend, getValue, setValue, initEmbedded, initUsdpl, targetUsdpl, versionUsdpl };

// helpers
export type BackendObject<T extends keyof BackendTypes> = { [K in T & keyof BackendTypes]: BackendTypes[K] };
