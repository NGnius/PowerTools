import { call_backend, set_value, get_value } from "usdpl-front";

// this module re-exports "usdpl-front" functions "call_backend", "set_value",
// and "get_value" with narrower types asserted

export type MinMax = {
    min: number | null;
    max: number | null;
};

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

export type GeneralLimits = never;

export type GpuLimits = {
    fast_ppt_limits: RangeLimit | null;
    slow_ppt_limits: RangeLimit | null;
    ppt_step: number;
    clock_min_limits: RangeLimit | null;
    clock_max_limits: RangeLimit | null;
    clock_step: number;
    memory_control_capable: boolean;
};

type ApiMessage<T extends unknown[] = never[]> = T;
type ApiCall<T extends unknown[], U extends unknown[]> = { params: ApiMessage<T>; return: ApiMessage<U> };

export const enum Battery {
    ChargeDesign = "BATTERY_charge_design",
    ChargeFull = "BATTERY_charge_full",
    ChargeMode = "BATTERY_charge_mode",
    ChargeNow = "BATTERY_charge_now",
    ChargeRate = "BATTERY_charge_rate",
    CurrentNow = "BATTERY_current_now",
}
export const enum Cpu {
    Governor = "CPUs_governor",
    MaxClock = "CPUs_max_clock",
    MinClock = "CPUs_min_clock",
    MinmaxClocks = "CPUs_minmax_clocks",
    Online = "CPUs_online",
    Smt = "CPUs_SMT",
    StatusOnline = "CPUs_status_online",
}
export const enum Gpu {
    FastPpt = "GPU_fastPPT",
    MaxClock = "GPU_max_clock",
    MinClock = "GPU_min_clock",
    SlowMemory = "GPU_slow_memory",
    SlowPpt = "GPU_slowPPT",
}
export const enum General {
    Name = "GENERAL_name",
    Persistent = "GENERAL_persistent",
    LimitsAll = "LIMITS_all",
    VInfo = "V_INFO",
}

export const enum BackendCalls {
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

type CallBackendFrameworkMap = {
    [BackendCalls.BatteryChargeDesign]: ApiCall<never[], [number]>;
    [BackendCalls.BatteryChargeFull]: ApiCall<never[], [number]>;
    [BackendCalls.BatteryChargeNow]: ApiCall<never[], [number]>;
    [BackendCalls.BatteryCurrentNow]: ApiCall<never[], [number]>;
    [BackendCalls.BatteryGetChargeMode]: ApiCall<never[], [string]>;
    [BackendCalls.BatteryGetChargeRate]: ApiCall<never[], [number]>;
    [BackendCalls.BatterySetChargeMode]: ApiCall<[string], [string]>;
    [BackendCalls.BatterySetChargeRate]: ApiCall<[rate: number], [number]>;
    [BackendCalls.BatteryUnsetChargeMode]: ApiCall<never[], unknown[]>;
    [BackendCalls.BatteryUnsetChargeRate]: ApiCall<never[], unknown[]>;
    [BackendCalls.CpuCount]: ApiCall<never[], [number]>;
    [BackendCalls.CpuGetClockLimits]: ApiCall<[index: number], [number, number]>;
    [BackendCalls.CpuGetGovernors]: ApiCall<never[], string[]>;
    [BackendCalls.CpuGetOnlines]: ApiCall<never[], boolean[]>;
    [BackendCalls.CpuSetClockLimits]: ApiCall<[index: number, min: number, max: number], [number, number]>;
    [BackendCalls.CpuSetGovernor]: ApiCall<[index: number, val: string], [string]>;
    [BackendCalls.CpuSetOnline]: ApiCall<[index: number, online: boolean], [boolean]>;
    [BackendCalls.CpuSetOnlines]: ApiCall<boolean[], boolean[]>;
    [BackendCalls.CpuSetSmt]: ApiCall<[status: boolean], [boolean]>;
    [BackendCalls.CpuUnsetClockLimits]: ApiCall<[index: number], unknown[]>;
    [BackendCalls.GeneralGetLimits]: ApiCall<never[], [SettingsLimits]>;
    [BackendCalls.GeneralGetName]: ApiCall<never[], [string]>;
    [BackendCalls.GeneralGetPersistent]: ApiCall<never[], [boolean]>;
    [BackendCalls.GeneralIdk]: ApiCall<unknown[], unknown[]>;
    [BackendCalls.GeneralLoadDefaultSettings]: ApiCall<never[], [boolean]>;
    [BackendCalls.GeneralLoadSettings]: ApiCall<[path: string, name: string], [boolean]>;
    [BackendCalls.GeneralLoadSystemSettings]: ApiCall<never[], boolean[]>;
    [BackendCalls.GeneralSetPersistent]: ApiCall<[boolean], [boolean]>;
    [BackendCalls.GeneralWaitForUnlocks]: ApiCall<never[], unknown[]>;
    [BackendCalls.GpuGetClockLimits]: ApiCall<never[], [number, number]>;
    [BackendCalls.GpuGetPpt]: ApiCall<never[], [number, number]>;
    [BackendCalls.GpuGetSlowMemory]: ApiCall<never[], [boolean]>;
    [BackendCalls.GpuSetClockLimits]: ApiCall<[min: number, max: number], [number, number]>;
    [BackendCalls.GpuSetPpt]: ApiCall<[fast: number, slow: number], [number, number]>;
    [BackendCalls.GpuSetSlowMemory]: ApiCall<[boolean], [boolean]>;
    [BackendCalls.GpuUnsetClockLimits]: ApiCall<never[], unknown[]>;
    [BackendCalls.GpuUnsetPpt]: ApiCall<never[], unknown[]>;
    [BackendCalls.VInfo]: ApiCall<never[], [string]>;
};

export type BackendFrameworkMap = {
    [Battery.ChargeDesign]: number;
    [Battery.ChargeFull]: number;
    [Battery.ChargeNow]: number;
    [Battery.ChargeRate]: number | null;
    [Battery.CurrentNow]: number;
    [Battery.ChargeMode]: string | null;
    [Cpu.Governor]: string[];
    [Cpu.MaxClock]: number | null;
    [Cpu.MinClock]: number | null;
    [Cpu.MinmaxClocks]: MinMax[];
    [Cpu.Online]: number;
    [Cpu.Smt]: boolean;
    [Cpu.StatusOnline]: boolean[];
    [Gpu.FastPpt]: number | null;
    [Gpu.MaxClock]: number | null;
    [Gpu.MinClock]: number | null;
    [Gpu.SlowMemory]: boolean;
    [Gpu.SlowPpt]: number | null;
    [General.Name]: string;
    [General.Persistent]: boolean;
    [General.LimitsAll]: SettingsLimits;
    [General.VInfo]: string;
};

type SetValue = <T extends keyof BackendFrameworkMap>(key: T, value: BackendFrameworkMap[T]) => unknown;

type GetValue = <T extends keyof BackendFrameworkMap>(key: T) => BackendFrameworkMap[T];

type CallBackend = <K extends keyof CallBackendFrameworkMap>(
    name: K,
    parameters: CallBackendFrameworkMap[K]["params"]
) => Promise<CallBackendFrameworkMap[K]["return"]>;

export const setValue = set_value as SetValue;
export const getValue = get_value as GetValue;
export const callBackend = call_backend as CallBackend;
