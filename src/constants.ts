// usdpl persistent store keys

import { KVMap } from "./utilities/augmentedUsdplFront";

type FilterOnPrefix<Prefix extends string, T> = T extends `${Prefix}${string}` ? T : never;

export type General = FilterOnPrefix<"GENERAL", keyof KVMap> | Extract<keyof KVMap, "V_INFO">;
export type Battery = FilterOnPrefix<"BATTERY", keyof KVMap>;
export type Cpu = FilterOnPrefix<"CPU", keyof KVMap> | Extract<keyof KVMap, "LIMITS_all">;
export type Gpu = FilterOnPrefix<"GPU", keyof KVMap> | Extract<keyof KVMap, "LIMITS_all">;

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
