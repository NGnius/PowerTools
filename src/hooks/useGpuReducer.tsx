import { Gpu, General, BackendCalls, callBackend, getValue } from "../usdplFront";
import { useAsyncReducer } from "../hooks/useAsyncReducer";
import { backendFactory, clone } from "../utilities/backendFactory";

type Action =
    | [type: "fastPPT", payload: number]
    | [type: "freqToggle", payload: boolean]
    | [type: "maxClock", payload: number]
    | [type: "minClock", payload: number]
    | [type: "pptToggle", payload: boolean]
    | [type: "slowMemory", payload: boolean]
    | [type: "slowPPT", payload: number]
    | [type: "unsetFreq", payload?: undefined]
    | [type: "unsetPPT", payload?: undefined];

const getInitialState = () => backendFactory([Gpu.FastPpt, Gpu.SlowPpt, Gpu.MinClock, Gpu.MaxClock, Gpu.SlowMemory]);

type State = ReturnType<typeof getInitialState>;

async function reducer(state: State, action: Action) {
    const [type, payload] = action;
    const limits = getValue(General.LimitsAll);
    const { slow_ppt_limits, fast_ppt_limits } = limits.gpu;

    console.debug(`GPU Action: ${type}; Payload: ${payload}`);

    switch (type) {
        case "slowMemory": {
            const [val] = await callBackend(BackendCalls.GpuSetSlowMemory, [payload]);
            state.GPU_slow_memory = val;
            const slowMemory = state.GPU_slow_memory;
            return slowMemory === state.GPU_slow_memory ? state : clone(state);
        }
        case "maxClock": {
            const maxNow = state.GPU_max_clock;
            const minNow = state.GPU_min_clock;
            if (payload !== maxNow && minNow) {
                const limits = await callBackend(BackendCalls.GpuSetClockLimits, [minNow, payload]); // -> [min, max]
                state.GPU_min_clock = limits[0];
                state.GPU_max_clock = limits[1];
            }
            const clockMin = state.GPU_min_clock;
            const clockMax = state.GPU_max_clock;
            return clockMin === state.GPU_min_clock && clockMax === state.GPU_max_clock ? state : clone(state);
        }
        case "minClock": {
            const minNow = state.GPU_min_clock;
            const maxNow = state.GPU_max_clock;
            if (payload !== minNow && maxNow) {
                const limits = await callBackend(BackendCalls.GpuSetClockLimits, [payload, maxNow]); // -> [min, max]
                state.GPU_min_clock = limits[0];
                state.GPU_max_clock = limits[1];
            }
            const clockMin = state.GPU_min_clock;
            const clockMax = state.GPU_max_clock;
            return clockMin === state.GPU_min_clock && clockMax === state.GPU_max_clock ? state : clone(state);
        }
        case "freqToggle": {
            if (payload) {
                const clock_min_limits = limits.gpu.clock_min_limits;
                const clock_max_limits = limits.gpu.clock_max_limits;
                if (clock_min_limits !== null) {
                    state.GPU_min_clock = clock_min_limits.min;
                }
                if (clock_max_limits !== null) {
                    state.GPU_max_clock = clock_max_limits.max;
                }
            } else {
                state.GPU_min_clock = null;
                state.GPU_max_clock = null;
                await callBackend(BackendCalls.GpuUnsetClockLimits, []);
            }
            const clockMin = state.GPU_min_clock;
            const clockMax = state.GPU_max_clock;
            return clockMin === state.GPU_min_clock && clockMax === state.GPU_max_clock ? state : clone(state);
        }
        case "fastPPT": {
            const pptNow = state.GPU_fastPPT;
            const realPpt = payload;
            // is GPU_slowPPT null allowed?
            if (realPpt !== pptNow && state.GPU_slowPPT !== null) {
                const limits: number[] = await callBackend(BackendCalls.GpuSetPpt, [realPpt, state.GPU_slowPPT]); // -> [fastPPT, slowPPT]
                state.GPU_fastPPT = limits[0];
                state.GPU_slowPPT = limits[1];
            }
            const fastPpt = state.GPU_fastPPT;
            const slowPpt = state.GPU_slowPPT;
            return fastPpt === state.GPU_fastPPT && slowPpt === state.GPU_slowPPT ? state : clone(state);
        }
        case "slowPPT": {
            const pptNow = state.GPU_slowPPT;
            const realPpt = payload;
            // is GPU_fastPPT null allowed?
            if (realPpt !== pptNow && state.GPU_fastPPT !== null) {
                const limits: number[] = await callBackend(BackendCalls.GpuSetPpt, [state.GPU_fastPPT, payload]); // -> [fastPPT, slowPPT]
                state.GPU_fastPPT = limits[0];
                state.GPU_slowPPT = limits[1];
            }
            const fastPpt = state.GPU_fastPPT;
            const slowPpt = state.GPU_slowPPT;
            return fastPpt === state.GPU_fastPPT && slowPpt === state.GPU_slowPPT ? state : clone(state);
        }
        case "pptToggle": {
            if (payload) {
                if (slow_ppt_limits) {
                    state.GPU_slowPPT = slow_ppt_limits.max;
                }
                if (fast_ppt_limits) {
                    state.GPU_fastPPT = fast_ppt_limits.max;
                }
            } else {
                state.GPU_slowPPT = null;
                state.GPU_fastPPT = null;
                await callBackend(BackendCalls.GpuUnsetPpt, []);
            }
            return clone(state);
        }
        default:
            throw new Error(`Unhandled GPU action ${type}`);
    }
}

export const useGpuReducer = () => useAsyncReducer(reducer, getInitialState);
