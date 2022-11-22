import { call_backend, set_value } from "./augmentedUsdplFront";
import { countCpus } from "./countCpus";
import { syncPlebClockToAdvanced } from "./syncPlebClockToAdvanced";

export async function reload(usdplReady: boolean, smtAllowedRef: { current: boolean }): Promise<void> {
    if (!usdplReady) {
        return;
    }
    await Promise.all([
        call_backend("GENERAL_get_limits", []).then(([limits]) => {
            set_value("LIMITS_all", limits);
            console.debug("POWERTOOLS: got limits", limits);
        }),
        call_backend("BATTERY_current_now", []).then(([rate]) => set_value("BATTERY_current_now", rate)),
        call_backend("BATTERY_get_charge_rate", []).then(([rate]) => set_value("BATTERY_charge_rate", rate)),
        call_backend("BATTERY_charge_now", []).then(([rate]) => set_value("BATTERY_charge_now", rate)),
        call_backend("BATTERY_charge_full", []).then(([rate]) => set_value("BATTERY_charge_full", rate)),
        call_backend("BATTERY_charge_design", []).then(([rate]) => set_value("BATTERY_charge_design", rate)),
        // call_backend("CPU_count", []).then(([count]) => set_value("CPUs_total", count)),
        call_backend("CPU_get_onlines", []).then((statii) => {
            set_value("CPUs_status_online", statii);
            set_value("CPUs_online", countCpus(statii));
            set_value("CPUs_SMT", statii.length > 3 && statii[0] === statii[1] && statii[2] === statii[3]);
            set_value(
                "CPUs_SMT",
                statii.length > 3 && statii[0] === statii[1] && statii[2] === statii[3] && smtAllowedRef.current
            );
        }),
        call_backend("CPU_get_clock_limits", [0]).then((limits) => {
            set_value("CPUs_max_clock", limits[0]);
            set_value("CPUs_max_clock", limits[1]);
            syncPlebClockToAdvanced();
        }),
        call_backend("CPU_get_governors", []).then((governors) => set_value("CPUs_governor", governors)),
        call_backend("GPU_get_ppt", []).then((ppts) => {
            set_value("GPU_fastPPT", ppts[0]);
            set_value("GPU_slowPPT", ppts[1]);
        }),
        call_backend("GPU_get_clock_limits", []).then((limits) => {
            set_value("GPU_min_clock", limits[0]);
            set_value("GPU_max_clock", limits[1]);
        }),
        call_backend("GPU_get_slow_memory", []).then(([status]) => set_value("GPU_slow_memory", status)),
        call_backend("GENERAL_get_persistent", []).then(([value]) => set_value("GENERAL_persistent", value)),
        call_backend("GENERAL_get_name", []).then(([name]) => set_value("GENERAL_name", name)),
        call_backend("V_INFO", []).then(([info]) => set_value("V_INFO", info)),
    ]);
}
