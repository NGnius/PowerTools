import { BACKEND_CALLS, callBackend, CpuTypes } from "../usdpl";
import { useAsyncReducer } from "../hooks/useAsyncReducer";
import { backendFactory, Copy } from "../utilities/backendFactory";
import { syncPlebClockToAdvanced } from "../utilities/syncPlebClockToAdvanced";
import { SETTINGS_LIMITS } from "../utilities/settingsLimits";
import { countCpus, notNull } from "../utilities/helpers";
import { CPU_DEFAULTS } from "../usdplBackend";

type Action =
    | [type: "refresh"]
    | [type: "advancedModeCpuSelector", payload: number]
    | [type: "advancedModeToggle", payload: boolean]
    | [type: "freqToggle", payload: boolean]
    | [type: "freqToggleAdvanced", payload: boolean]
    | [type: "governor", payload: string]
    | [type: "maxFreq", payload: number]
    | [type: "maxFreqAdvanced", payload: number]
    | [type: "minFreq", payload: number]
    | [type: "minFreqAdvanced", payload: number]
    | [type: "immediate", payload: number]
    | [type: "setSmt", payload: boolean]
    | [type: "setSmtAdvanced", payload: boolean];

type FeState = { advancedMode: boolean; advancedCpuIndex: number; smtAllowed: boolean; total_cpus: number };
// DO NOT MERGE WITH CPUS = 8

const combinedProperties = backendFactory({
    ...CPU_DEFAULTS,
    advancedMode: false,
    advancedCpuIndex: 0,
    smtAllowed: false,
    total_cpus: 8,
});

async function reducer(state: CpuTypes & FeState, action: Action): Promise<CpuTypes & FeState> {
    const [type, payload] = action;
    const { clock_min_limits, clock_max_limits } = SETTINGS_LIMITS.cpu.cpus[0];

    console.debug(`CPU Action: ${type}; Payload: ${payload}`);

    switch (type) {
        case "advancedModeToggle":
            state.advancedMode = payload;
            return state[Copy]();
        case "advancedModeCpuSelector":
            state.advancedCpuIndex = payload;
            return state[Copy]();
        case "governor": {
            const prevGov = state?.CPUs_governor?.[state.advancedCpuIndex];
            const [gov] = await callBackend(BACKEND_CALLS.CpuSetGovernor, [state.advancedCpuIndex, payload]);
            const governors = state.CPUs_governor;
            governors[state.advancedCpuIndex] = gov;
            state.CPUs_governor = governors;
            return state.CPUs_governor[state.advancedCpuIndex] === prevGov ? state : state[Copy]();
        }
        case "maxFreqAdvanced": {
            const { min, max } = state.CPUs_minmax_clocks[state.advancedCpuIndex];
            if (payload !== max && notNull(min)) {
                const limits = await callBackend(BACKEND_CALLS.CpuSetClockLimits, [
                    state.advancedCpuIndex,
                    min,
                    payload,
                ]);
                const clocks = state.CPUs_minmax_clocks;
                clocks[state.advancedCpuIndex].min = limits[0];
                clocks[state.advancedCpuIndex].max = limits[1];
                state.CPUs_minmax_clocks = clocks;
            }
            return state[Copy]();
        }
        case "maxFreq": {
            const freqNow = state.CPUs_max_clock;
            const minNow = state.CPUs_min_clock;
            if (payload !== freqNow && notNull(minNow)) {
                state.CPUs_max_clock = payload;
                for (let i = 0; i < state.total_cpus; i++) {
                    const limits = await callBackend(BACKEND_CALLS.CpuSetClockLimits, [i, minNow, payload]);
                    state.CPUs_min_clock = limits[0];
                    state.CPUs_max_clock = limits[1];
                    syncPlebClockToAdvanced();
                }
                await callBackend(BACKEND_CALLS.GeneralWaitForUnlocks, []);
            }
            return state[Copy]();
        }
        case "minFreqAdvanced": {
            const freqNow = state.CPUs_minmax_clocks[state.advancedCpuIndex];
            if (payload !== freqNow.min && notNull(freqNow.max)) {
                const limits = await callBackend(BACKEND_CALLS.CpuSetClockLimits, [
                    state.advancedCpuIndex,
                    payload,
                    freqNow.max,
                ]);
                const clocks = state.CPUs_minmax_clocks;
                clocks[state.advancedCpuIndex].min = limits[0];
                clocks[state.advancedCpuIndex].max = limits[1];
                state.CPUs_minmax_clocks = clocks;
            }
            return state[Copy]();
        }
        case "minFreq": {
            const freqNow = state.CPUs_min_clock;
            const maxNow = state.CPUs_max_clock;
            if (payload !== freqNow && notNull(maxNow)) {
                state.CPUs_min_clock = payload;
                for (let i = 0; i < state.total_cpus; i++) {
                    const limits = await callBackend(BACKEND_CALLS.CpuSetClockLimits, [i, payload, maxNow]);
                    state.CPUs_min_clock = limits[0];
                    state.CPUs_max_clock = limits[1];
                    syncPlebClockToAdvanced();
                }
                await callBackend(BACKEND_CALLS.GeneralWaitForUnlocks, []);
            }
            return state[Copy]();
        }
        case "freqToggleAdvanced": {
            if (payload) {
                if (notNull(clock_min_limits)) {
                    state.CPUs_min_clock = clock_min_limits.min;
                }
                if (notNull(clock_max_limits)) {
                    state.CPUs_max_clock = clock_max_limits.max;
                }
            } else {
                state.CPUs_minmax_clocks[state.advancedCpuIndex].min = null;
                state.CPUs_minmax_clocks[state.advancedCpuIndex].max = null;
                await callBackend(BACKEND_CALLS.CpuUnsetClockLimits, [state.advancedCpuIndex]);
            }
            return state[Copy]();
        }
        case "freqToggle": {
            if (payload) {
                if (notNull(clock_min_limits)) {
                    state.CPUs_minmax_clocks[state.advancedCpuIndex].min = clock_min_limits.min;
                }
                if (notNull(clock_max_limits)) {
                    state.CPUs_minmax_clocks[state.advancedCpuIndex].max = clock_max_limits.max;
                }
                syncPlebClockToAdvanced();
            } else {
                state.CPUs_min_clock = null;
                state.CPUs_max_clock = null;
                for (let i = 0; i < state.total_cpus; i++) {
                    // await unsetCpuClockLimits(i);
                    await callBackend(BACKEND_CALLS.CpuUnsetClockLimits, [i]);
                }
                await callBackend(BACKEND_CALLS.GeneralWaitForUnlocks, []);
                syncPlebClockToAdvanced();
            }
            return state[Copy]();
        }
        case "immediate": {
            const onlines = state.CPUs_online;
            if (payload !== onlines) {
                state.CPUs_online = payload;
                const smtNow = state.CPUs_SMT;
                const onlines: boolean[] = [];
                for (let i = 0; i < state.total_cpus; i++) {
                    const online = smtNow ? i < payload : i % 2 === 0 && i < payload * 2;
                    onlines.push(online);
                }
                const statii = await callBackend(BACKEND_CALLS.CpuSetOnlines, onlines);
                state.CPUs_status_online = statii;
                const count = countCpus(statii);
                state.CPUs_online = count;
            }
            return state[Copy]();
        }
        case "setSmtAdvanced": {
            const smtNow = state.CPUs_SMT;
            if (smtNow) {
                const [newSmt] = await callBackend(BACKEND_CALLS.CpuSetSmt, [false]);
                state.CPUs_SMT = newSmt;
            }
            const [onlineCpu] = await callBackend(BACKEND_CALLS.CpuSetOnline, [state.advancedCpuIndex, payload]);
            state.CPUs_status_online[state.advancedCpuIndex] = onlineCpu;
            return state[Copy]();
        }
        case "setSmt": {
            const total_cpus = state.total_cpus;
            const cpus = state.CPUs_online;
            const smtNow = payload && !!state.smtAllowed;
            const [newVal] = await callBackend(BACKEND_CALLS.CpuSetSmt, [smtNow]);
            state.CPUs_SMT = newVal;
            const onlines: boolean[] = [];
            for (let i = 0; i < total_cpus; i++) {
                const online = (smtNow ? i < cpus : i % 2 === 0 && i < cpus * 2) || (!smtNow && cpus === 4);
                onlines.push(online);
            }
            const statii = await callBackend(BACKEND_CALLS.CpuSetOnlines, onlines);
            state.CPUs_status_online = statii;
            state.CPUs_online = countCpus(statii);
            return state[Copy]();
        }
        case "refresh":
            return state[Copy]();
        default:
            throw new Error(`Unhandled CPU action ${type}`);
    }
}

export const useCpuReducer = () => useAsyncReducer(reducer, () => combinedProperties);
