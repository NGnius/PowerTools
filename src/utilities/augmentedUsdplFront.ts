// augments src/usdpl_front/usdpl-front.d.ts
// use module augmentation to make augmented types opt-in/opt-out by changing import in usdpl-front consumers

import { call_backend as _call_backend, get_value as _get_value, set_value as _set_value } from "usdpl-front";
import { MinMax } from "../types";

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


type BackendsTypeGuard<T extends Required<{ [K in Backends]: { params: any; return: any } }>> = T;

type CallBackendMap = {
    // TODO
    BATTERY_unset_charge_mode: { params: any[]; return: any[] };
    CPU_count: { params: any[]; return: any[] };
    GENERAL_idk: { params: any[]; return: any[] };
    //
    // CPU_count: { params: never[]; return: number[] };
    BATTERY_charge_design: { params: never[]; return: number[] };
    BATTERY_charge_full: { params: never[]; return: number[] };
    BATTERY_charge_now: { params: never[]; return: number[] };
    BATTERY_current_now: { params: never[]; return: number[] };
    BATTERY_get_charge_mode: { params: never[]; return: [string] };
    BATTERY_get_charge_rate: { params: never[]; return: number[] };
    BATTERY_set_charge_mode: { params: [string]; return: [string] };
    BATTERY_set_charge_rate: { params: [rate: number]; return: number[] };
    BATTERY_unset_charge_rate: { params: never[]; return: any[] };
    CPU_get_clock_limits: { params: [index: number]; return: number[] };
    CPU_get_governors: { params: never[]; return: string[] };
    CPU_get_onlines: { params: never[]; return: boolean[] };
    CPU_set_clock_limits: { params: [index: number, min: number, max: number]; return: number[] };
    CPU_set_governor: { params: [index: number, val: string]; return: string[] };
    CPU_set_online: { params: [index: number, online: boolean]; return: boolean[] };
    CPU_set_onlines: { params: boolean[]; return: boolean[] };
    CPU_set_smt: { params: [status: boolean]; return: boolean[] };
    CPU_unset_clock_limits: { params: [index: number]; return: any[] };
    GENERAL_get_limits: { params: never[]; return: [SettingsLimits] };
    GENERAL_get_name: { params: never[]; return: string[] };
    GENERAL_get_persistent: { params: never[]; return: boolean[] };
    GENERAL_load_default_settings: { params: never[]; return: boolean[] };
    GENERAL_load_settings: { params: [path: string, name: string]; return: boolean[] };
    GENERAL_load_system_settings: { params: never[]; return: boolean[] };
    GENERAL_set_persistent: { params: [boolean]; return: boolean[] };
    GENERAL_wait_for_unlocks: { params: never[]; return: boolean[] };
    GPU_get_clock_limits: { params: never[]; return: number[] };
    GPU_get_ppt: { params: never[]; return: number[] };
    GPU_get_slow_memory: { params: never[]; return: boolean[] };
    GPU_set_clock_limits: { params: [min: number, max: number]; return: number[] };
    GPU_set_ppt: { params: [fast: number, slow: number]; return: number[] };
    GPU_set_slow_memory: { params: [boolean]; return: boolean[] };
    GPU_unset_clock_limits: { params: never[]; return: any[] };
    GPU_unset_ppt: { params: never[]; return: any[] };
    V_INFO: { params: never[]; return: string[] };
}

export type BackendProperyMap = {
    BATTERY_charge_design: number;
    BATTERY_charge_full: number;
    BATTERY_charge_now: number;
    BATTERY_charge_rate: number | null;
    BATTERY_current_now: number;
    BATTERY_charge_mode: string | null;
    // CPUs_total: number;
    CPUs_governor: string[];
    CPUs_max_clock: number | null;
    CPUs_min_clock: number | null;
    CPUs_minmax_clocks: MinMax[];
    CPUs_online: number;
    CPUs_SMT: boolean;
    CPUs_status_online: boolean[];
    GENERAL_name: string;
    GENERAL_persistent: boolean;
    GPU_fastPPT: number;
    GPU_max_clock: number | null;
    GPU_min_clock: number | null;
    GPU_slow_memory: boolean;
    GPU_slowPPT: number;
    LIMITS_all: SettingsLimits;
    V_INFO: string;
    // VINFO: string;??
}

export type BackendProperties = keyof BackendProperyMap;

type GetValue = <T extends keyof BackendProperyMap>(key: T) => BackendProperyMap[T];

type SetValue = <T extends keyof BackendProperyMap>(key: T, value: BackendProperyMap[T]) => void;

type CallBackend = <K extends keyof CallBackendMap>(
    name: K,
    parameters: CallBackendMap[K]["params"]
) => Promise<CallBackendMap[K]["return"]>;

export const get_value: GetValue = _get_value;
export const set_value: SetValue = _set_value;
export const call_backend: CallBackend = _call_backend;
