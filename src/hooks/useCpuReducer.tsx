import { BackendCalls, BackendFrameworkMap, Cpu, General, callBackend, getValue } from "../usdplFront";
import { useAsyncReducer } from "../hooks/useAsyncReducer";
import { assertRequired } from "../utilities/assertRequired";
import { backendFactory, clone } from "../utilities/backendFactory";
import { countCpus } from "../utilities/countCpus";
import { syncPlebClockToAdvanced } from "../utilities/syncPlebClockToAdvanced";

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

const bePropertyNames = [
    Cpu.Online,
    Cpu.StatusOnline,
    Cpu.Smt,
    Cpu.MinClock,
    Cpu.MaxClock,
    Cpu.MinmaxClocks,
    Cpu.Governor,
] as const;

type BeState = Pick<BackendFrameworkMap, typeof bePropertyNames[number]>;
type FeState = { advancedMode: boolean; advancedCpuIndex: number; smtAllowed: boolean; total_cpus: number };

const localPropertyNames = ["advancedMode", "advancedCpuIndex", "smtAllowed", "total_cpus"] as const;
const allPropertyNames = [...bePropertyNames, ...localPropertyNames];

function getDerivedState(state: BeState & Partial<FeState>): BeState & FeState {
    // initialize FE state
    const limits = getValue(General.LimitsAll);
    const total_cpus = limits.cpu.count ?? 8;

    if (state.smtAllowed !== state.smtAllowed || total_cpus !== state.total_cpus) {
        state.total_cpus = total_cpus;
    }

    state.smtAllowed = limits.cpu.smt_capable ?? false;
    state.advancedMode = state.advancedMode ?? false;
    state.advancedCpuIndex = state.advancedCpuIndex ?? 0;
    assertRequired(state, allPropertyNames);
    return state;
}
const getInitialState = () => getDerivedState(backendFactory(bePropertyNames));

type State = ReturnType<typeof getInitialState>;

async function reducer(state: State, action: Action): Promise<State> {
    const [type, payload] = action;
    const limits = getValue(General.LimitsAll);
    const { clock_min_limits, clock_max_limits } = limits.cpu.cpus[0];

    console.debug(`CPU Action: ${type}; Payload: ${payload}`);

    switch (type) {
        case "advancedModeToggle":
            state.advancedMode = payload;
            return clone(state);
        case "advancedModeCpuSelector":
            state.advancedCpuIndex = payload;
            return clone(state);
        case "governor": {
            const prevGov = state.CPUs_governor[state.advancedCpuIndex];
            const [gov] = await callBackend(BackendCalls.CpuSetGovernor, [state.advancedCpuIndex, payload]);
            const governors = state.CPUs_governor;
            governors[state.advancedCpuIndex] = gov;
            state.CPUs_governor = governors;
            return state.CPUs_governor[state.advancedCpuIndex] === prevGov ? state : clone(state);
        }
        case "maxFreqAdvanced": {
            const { min, max } = state.CPUs_minmax_clocks[state.advancedCpuIndex];
            if (payload !== max && min !== null) {
                const limits = await callBackend(BackendCalls.CpuSetClockLimits, [
                    state.advancedCpuIndex,
                    min,
                    payload,
                ]);
                const clocks = state.CPUs_minmax_clocks;
                clocks[state.advancedCpuIndex].min = limits[0];
                clocks[state.advancedCpuIndex].max = limits[1];
                state.CPUs_minmax_clocks = clocks;
            }
            return clone(state);
        }
        case "maxFreq": {
            const freqNow = state.CPUs_max_clock;
            const minNow = state.CPUs_min_clock;
            if (payload !== freqNow && minNow) {
                state.CPUs_max_clock = payload;
                for (let i = 0; i < state.total_cpus; i++) {
                    const limits = await callBackend(BackendCalls.CpuSetClockLimits, [i, minNow, payload]);
                    state.CPUs_min_clock = limits[0];
                    state.CPUs_max_clock = limits[1];
                    syncPlebClockToAdvanced();
                }
                await callBackend(BackendCalls.GeneralWaitForUnlocks, []);
            }
            return clone(state);
        }
        case "minFreqAdvanced": {
            const freqNow = state.CPUs_minmax_clocks[state.advancedCpuIndex];
            if (payload !== freqNow.min && freqNow.max !== null) {
                const limits = await callBackend(BackendCalls.CpuSetClockLimits, [
                    state.advancedCpuIndex,
                    payload,
                    freqNow.max,
                ]);
                const clocks = state.CPUs_minmax_clocks;
                clocks[state.advancedCpuIndex].min = limits[0];
                clocks[state.advancedCpuIndex].max = limits[1];
                state.CPUs_minmax_clocks = clocks;
            }
            return clone(state);
        }
        case "minFreq": {
            const freqNow = state.CPUs_min_clock;
            const maxNow = state.CPUs_max_clock;
            if (payload !== freqNow && maxNow) {
                state.CPUs_min_clock = payload;
                for (let i = 0; i < state.total_cpus; i++) {
                    const limits = await callBackend(BackendCalls.CpuSetClockLimits, [i, payload, maxNow]);
                    state.CPUs_min_clock = limits[0];
                    state.CPUs_max_clock = limits[1];
                    syncPlebClockToAdvanced();
                }
                await callBackend(BackendCalls.GeneralWaitForUnlocks, []);
            }
            return clone(state);
        }
        case "freqToggleAdvanced": {
            if (payload) {
                if (clock_min_limits !== null) {
                    state.CPUs_min_clock = clock_min_limits.min;
                }
                if (clock_max_limits !== null) {
                    state.CPUs_max_clock = clock_max_limits.max;
                }
            } else {
                state.CPUs_minmax_clocks[state.advancedCpuIndex].min = null;
                state.CPUs_minmax_clocks[state.advancedCpuIndex].max = null;
                await callBackend(BackendCalls.CpuUnsetClockLimits, [state.advancedCpuIndex]);
            }
            return clone(state);
        }
        case "freqToggle": {
            if (payload) {
                if (clock_min_limits !== null) {
                    state.CPUs_minmax_clocks[state.advancedCpuIndex].min = clock_min_limits.min;
                }
                if (clock_max_limits !== null) {
                    state.CPUs_minmax_clocks[state.advancedCpuIndex].max = clock_max_limits.max;
                }
                syncPlebClockToAdvanced();
            } else {
                state.CPUs_min_clock = null;
                state.CPUs_max_clock = null;
                for (let i = 0; i < state.total_cpus; i++) {
                    // await unsetCpuClockLimits(i);
                    await callBackend(BackendCalls.CpuUnsetClockLimits, [i]);
                }
                await callBackend(BackendCalls.GeneralWaitForUnlocks, []);
                syncPlebClockToAdvanced();
            }
            return clone(state);
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
                const statii = await callBackend(BackendCalls.CpuSetOnlines, onlines);
                state.CPUs_status_online = statii;
                const count = countCpus(statii);
                state.CPUs_online = count;
            }
            return clone(state);
        }
        case "setSmtAdvanced": {
            const smtNow = state.CPUs_SMT;
            if (smtNow) {
                const [newSmt] = await callBackend(BackendCalls.CpuSetSmt, [false]);
                state.CPUs_SMT = newSmt;
            }
            const [onlineCpu] = await callBackend(BackendCalls.CpuSetOnline, [state.advancedCpuIndex, payload]);
            state.CPUs_status_online[state.advancedCpuIndex] = onlineCpu;
            return clone(state);
        }
        case "setSmt": {
            const total_cpus = state.total_cpus ?? -1;
            const cpus = state.CPUs_online;
            const smtNow = payload && !!state.smtAllowed;
            const [newVal] = await callBackend(BackendCalls.CpuSetSmt, [smtNow]);
            state.CPUs_SMT = newVal;
            const onlines: boolean[] = [];
            for (let i = 0; i < total_cpus; i++) {
                const online = (smtNow ? i < cpus : i % 2 === 0 && i < cpus * 2) || (!smtNow && cpus === 4);
                onlines.push(online);
            }
            const statii = await callBackend(BackendCalls.CpuSetOnlines, onlines);
            state.CPUs_status_online = statii;
            state.CPUs_online = countCpus(statii);
            return clone(state);
        }
        case "refresh":
            return clone(state);
        default:
            throw new Error(`Unhandled CPU action ${type}`);
    }
}

export const useCpuReducer = () => useAsyncReducer(reducer, getInitialState);
