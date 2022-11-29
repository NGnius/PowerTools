import { GpuTypes, GPU, CpuTypes, CPU, BatteryTypes, BATTERY, GENERAL, GeneralTypes, SettingsLimits } from "./usdpl";
import { backendFactory } from "./utilities/backendFactory";

import { fill } from "./utilities/helpers";

export const limits: SettingsLimits = {
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

export const GPU_DEFAULTS: GpuTypes = {
    [GPU.FastPpt]: null,
    [GPU.MaxClock]: null,
    [GPU.MinClock]: null,
    [GPU.SlowMemory]: false,
    [GPU.SlowPpt]: null,
};

export const CPU_DEFAULTS: CpuTypes = {
    [CPU.Governor]: fill("").map((_, i) => i.toString()),
    [CPU.MaxClock]: null,
    [CPU.MinClock]: null,
    [CPU.MinmaxClocks]: fill({ min: null, max: null }),
    [CPU.Online]: 8,
    [CPU.Smt]: false,
    [CPU.StatusOnline]: [true, true, true, true, false, false, false, false],
};

export const BATTERY_DEFAULTS: BatteryTypes = {
    [BATTERY.ChargeDesign]: 0,
    [BATTERY.ChargeFull]: 0,
    [BATTERY.ChargeMode]: null,
    [BATTERY.ChargeNow]: 0,
    [BATTERY.ChargeRate]: null,
    [BATTERY.CurrentNow]: 0,
};

export const GENERAL_DEFAULTS: GeneralTypes = {
    [GENERAL.LimitsAll]: limits,
    [GENERAL.Name]: "NAME",
    [GENERAL.Persistent]: false,
    [GENERAL.VInfo]: "V_INFO",
};

export const BATTERY_BE = backendFactory(BATTERY_DEFAULTS);
export const CPU_BE = backendFactory(CPU_DEFAULTS);
export const GENERAL_BE = backendFactory(GENERAL_DEFAULTS);
export const GPU_BE = backendFactory(GPU_DEFAULTS);
