import { BackendProperyMap, BackendProperties } from "./utilities/augmentedUsdplFront";

type FilterOnPrefix<Prefix extends string, T> = T extends `${Prefix}${string}` ? T : never;
type Limits = Extract<BackendProperties, "LIMITS_all">

export type General = FilterOnPrefix<"GENERAL", BackendProperties> | Extract<BackendProperties, "V_INFO">;
export type Battery = FilterOnPrefix<"BATTERY", BackendProperties>;
export type Cpu = FilterOnPrefix<"CPU", BackendProperties> | Limits;
export type Gpu = FilterOnPrefix<"GPU", BackendProperties> | Limits;

export const GENERAL: General[] = ["V_INFO", "GENERAL_persistent", "GENERAL_name"];

export const BATTERY: Battery[] = [
    "BATTERY_current_now",
    "BATTERY_charge_rate",
    "BATTERY_charge_now",
    "BATTERY_charge_full",
    "BATTERY_charge_design",
    "BATTERY_charge_mode",
];

export const CPU: Cpu[] = [
    // "CPUs_total",
    "LIMITS_all",
    "CPUs_online",
    "CPUs_status_online",
    "CPUs_SMT",
    "CPUs_min_clock",
    "CPUs_max_clock",
    "CPUs_minmax_clocks",
    "CPUs_governor",
];

export const GPU: Gpu[] = [
    "LIMITS_all",
    "GPU_fastPPT",
    "GPU_slowPPT",
    "GPU_min_clock",
    "GPU_max_clock",
    "GPU_slow_memory",
];
