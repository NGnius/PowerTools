import { Cpu, CPU } from "../constants";
import { useAsyncReducer } from "../hooks/useAsyncReducer";
import { MinMax } from "../types";
import { call_backend } from "../utilities/augmentedUsdplFront";
import { backendFactory, BackendObject, clone } from "../utilities/backendFactory";
import { countCpus } from "../utilities/countCpus";
import { syncPlebClockToAdvanced } from "../utilities/syncPlebClockToAdvanced";

type Action =
    | [type: "refresh"]
    | [type: "advancedModeCpuSelector", payload: number]
    | [type: "advancedModeToggle", payload: boolean]
    | [type: "CPUFreqToggle", payload: boolean]
    | [type: "CPUFreqToggleAdvanced", payload: boolean]
    | [type: "CPUGovernor", payload: string]
    | [type: "CPUMaxFreq", payload: number]
    | [type: "CPUMaxFreqAdvanced", payload: number]
    | [type: "CPUMinFreq", payload: number]
    | [type: "CPUMinFreqAdvanced", payload: number]
    | [type: "CPUsImmediate", payload: number]
    | [type: "SMT", payload: boolean]
    | [type: "SMTAdvanced", payload: boolean];

const getInitialState = (): BackendObject<Cpu> & {
    advancedMode?: boolean;
    advancedModeCpu?: number;
    smtAllowed?: boolean;
    total_cpus?: number;
} => backendFactory(CPU);

type State = ReturnType<typeof getInitialState>;

async function reducer(_state: State, action: Action): Promise<State> {
    let state = _state;
    const { advancedMode = false, advancedModeCpu = 0 } = state;
    const [type, payload] = action;
    // const total_cpus = state.CPUs_total;
    const total_cpus = state.LIMITS_all?.cpu.count ?? 8;
    console.debug(`CPU Action: ${type}; Payload: ${payload}`);
    const smtAllowed = state.LIMITS_all?.cpu.smt_capable ?? true;
    if (smtAllowed !== state.smtAllowed || total_cpus !== state.total_cpus) {
        state.total_cpus = total_cpus;
        state.smtAllowed = smtAllowed;
        state = clone(state);
    }
    state.advancedMode = advancedMode;
    state.advancedModeCpu = advancedModeCpu;
    switch (type) {
        case "advancedModeToggle":
            state.advancedMode = payload;
            return clone(state);
        case "advancedModeCpuSelector":
            state.advancedModeCpu = payload;
            return clone(state);
        case "CPUGovernor": {
            const prevGov = state.CPUs_governor[advancedModeCpu];
            const [gov] = await call_backend("CPU_set_governor", [advancedModeCpu, payload]);
            const governors = state.CPUs_governor;
            governors[advancedModeCpu] = gov;
            state.CPUs_governor = governors;
            return state.CPUs_governor[advancedModeCpu] === prevGov ? state : clone(state);
        }
        case "CPUMaxFreqAdvanced": {
            const freqNow = state.CPUs_minmax_clocks[advancedModeCpu] as MinMax;
            if (payload !== freqNow.max) {
                // @ts-expect-error setCpuClockLimits expects number but freqNow
                const limits = await call_backend("CPU_set_clock_limits", [advancedModeCpu, freqNow.min, freq]);
                const clocks = state.CPUs_minmax_clocks as MinMax[];
                clocks[advancedModeCpu].min = limits[0];
                clocks[advancedModeCpu].max = limits[1];
                state.CPUs_minmax_clocks = clocks;
            }
            return clone(state);
        }
        case "CPUMaxFreq": {
            const freqNow = state.CPUs_max_clock;
            const minNow = state.CPUs_min_clock;
            if (payload !== freqNow && minNow) {
                state.CPUs_max_clock = payload;
                for (let i = 0; i < total_cpus; i++) {
                    const limits = await call_backend("CPU_set_clock_limits", [i, minNow, payload]);
                    state.CPUs_min_clock = limits[0];
                    state.CPUs_max_clock = limits[1];
                    syncPlebClockToAdvanced();
                }
                await call_backend("GENERAL_wait_for_unlocks", []);
            }
            return clone(state);
        }
        case "CPUMinFreqAdvanced": {
            const freqNow = state.CPUs_minmax_clocks[advancedModeCpu] as MinMax;
            if (payload !== freqNow.min) {
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                const limits = await call_backend("CPU_set_clock_limits", [advancedModeCpu, payload, freqNow.max!]);
                const clocks = state.CPUs_minmax_clocks as MinMax[];
                clocks[advancedModeCpu].min = limits[0];
                clocks[advancedModeCpu].max = limits[1];
                state.CPUs_minmax_clocks = clocks;
            }
            return clone(state);
        }
        case "CPUMinFreq": {
            const freqNow = state.CPUs_min_clock;
            const maxNow = state.CPUs_max_clock;
            if (payload !== freqNow && maxNow) {
                state.CPUs_min_clock = payload;
                for (let i = 0; i < total_cpus; i++) {
                    const limits = await call_backend("CPU_set_clock_limits", [i, payload, maxNow]);
                    state.CPUs_min_clock = limits[0];
                    state.CPUs_max_clock = limits[1];
                    syncPlebClockToAdvanced();
                }
                await call_backend("GENERAL_wait_for_unlocks", []);
            }
            return clone(state);
        }
        case "CPUFreqToggleAdvanced": {
            if (payload) {
                const clocks = state.CPUs_minmax_clocks;
                if (state.LIMITS_all.cpu.cpus[0].clock_min_limits !== null) {
                    state.CPUs_min_clock = state.LIMITS_all.cpu.cpus[0].clock_min_limits.min;
                }
                if (state.LIMITS_all.cpu.cpus[0].clock_max_limits !== null) {
                    state.CPUs_max_clock = state.LIMITS_all.cpu.cpus[0].clock_max_limits.max;
                }
                // clocks[advancedModeCpu].min = 1400;
                // clocks[advancedModeCpu].max = 3500;
                state.CPUs_minmax_clocks = clocks;
            } else {
                const clocks = state.CPUs_minmax_clocks;
                clocks[advancedModeCpu].min = null;
                clocks[advancedModeCpu].max = null;
                state.CPUs_minmax_clocks = clocks;
                await call_backend("CPU_unset_clock_limits", [advancedModeCpu]);
            }
            return clone(state);
        }
        case "CPUFreqToggle": {
            if (payload) {
                const clocks = state.CPUs_minmax_clocks;
                if (state.LIMITS_all.cpu.cpus[advancedModeCpu].clock_min_limits !== null) {
                    clocks[advancedModeCpu].min = state.LIMITS_all.cpu.cpus[advancedModeCpu].clock_min_limits.min;
                }
                if (state.LIMITS_all.cpu.cpus[advancedModeCpu].clock_max_limits !== null) {
                    clocks[advancedModeCpu].max = state.LIMITS_all.cpu.cpus[advancedModeCpu].clock_max_limits.max;
                }
                // state.CPUs_min_clock = 1400;
                // state.CPUs_max_clock = 3500;
                syncPlebClockToAdvanced();
            } else {
                state.CPUs_min_clock = null;
                state.CPUs_max_clock = null;
                for (let i = 0; i < total_cpus; i++) {
                    // await unsetCpuClockLimits(i);
                    await call_backend("CPU_unset_clock_limits", [i]);
                }
                await call_backend("GENERAL_wait_for_unlocks", []);
                syncPlebClockToAdvanced();
            }
            return clone(state);
        }
        case "CPUsImmediate": {
            const onlines = state.CPUs_online;
            if (payload !== onlines) {
                state.CPUs_online = payload;
                const smtNow = state.CPUs_SMT;
                const onlines: boolean[] = [];
                for (let i = 0; i < total_cpus; i++) {
                    const online = smtNow ? i < payload : i % 2 === 0 && i < payload * 2;
                    onlines.push(online);
                }
                const statii = await call_backend("CPU_set_onlines", onlines);
                state.CPUs_status_online = statii;
                const count = countCpus(statii);
                state.CPUs_online = count;
            }
            return clone(state);
        }
        case "SMTAdvanced": {
            const smtNow = state.CPUs_SMT;
            if (smtNow) {
                const [newSmt] = await call_backend("CPU_set_smt", [false]);
                state.CPUs_SMT = newSmt;
            }
            const [onlineCpu] = await call_backend("CPU_set_online", [advancedModeCpu, payload]);
            // will this work with setter/getter?
            state.CPUs_status_online[advancedModeCpu] = onlineCpu;
            return clone(state);
        }
        case "SMT": {
            const total_cpus = state.total_cpus ?? -1;
            const cpus = state.CPUs_online;
            const smtNow = payload && !!state.smtAllowed;
            const [newVal] = await call_backend("CPU_set_smt", [smtNow]);
            state.CPUs_SMT = newVal;
            const onlines: boolean[] = [];
            for (let i = 0; i < total_cpus; i++) {
                const online = (smtNow ? i < cpus : i % 2 === 0 && i < cpus * 2) || (!smtNow && cpus === 4);
                onlines.push(online);
            }
            const statii = await call_backend("CPU_set_onlines", onlines);
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
