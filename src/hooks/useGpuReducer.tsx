import { GPU } from "../constants";
import { useAsyncReducer } from "../hooks/useAsyncReducer";
import { call_backend } from "../utilities/augmentedUsdplFront";
import { backendFactory, clone } from "../utilities/backendFactory";

type Action =
    | [type: "GPUFastPPT", payload: number]
    | [type: "GPUFreqToggle", payload: boolean]
    | [type: "GPUMaxClock", payload: number]
    | [type: "GPUMinClock", payload: number]
    | [type: "GPUPPTToggle", payload: number | null]
    | [type: "GPUSlowMemory", payload: boolean]
    | [type: "GPUSlowPPT", payload: number]
    | [type: "GPUUnsetFreq", payload?: undefined]
    | [type: "GPUUnsetPPT", payload?: undefined];

const getInitialState = () => backendFactory(GPU);

type State = ReturnType<typeof getInitialState>;

async function reducer(state: State, action: Action) {
    const [type, payload] = action;
    console.debug(`GPU Action: ${type}; Payload: ${payload}`);
    if (type) {
        return state;
    }
    switch (type) {
        case "GPUSlowMemory": {
            const [val] = await call_backend("GPU_set_slow_memory", [payload]);
            state.GPU_slow_memory = val;
            const slowMemory = state.GPU_slow_memory;
            return slowMemory === state.GPU_slow_memory ? state : clone(state);
        }
        case "GPUMaxClock": {
            const maxNow = state.GPU_max_clock;
            const minNow = state.GPU_min_clock;
            if (payload !== maxNow && minNow) {
                const limits = await call_backend("GPU_set_clock_limits", [minNow, payload]); // -> [min, max]
                state.GPU_min_clock = limits[0];
                state.GPU_max_clock = limits[1];
            }
            const clockMin = state.GPU_min_clock;
            const clockMax = state.GPU_max_clock;
            return clockMin === state.GPU_min_clock && clockMax === state.GPU_max_clock ? state : clone(state);
        }
        case "GPUMinClock": {
            const minNow = state.GPU_min_clock;
            const maxNow = state.GPU_max_clock;
            if (payload !== minNow && maxNow) {
                const limits = await call_backend("GPU_set_clock_limits", [payload, maxNow]); // -> [min, max]
                state.GPU_min_clock = limits[0];
                state.GPU_max_clock = limits[1];
            }
            const clockMin = state.GPU_min_clock;
            const clockMax = state.GPU_max_clock;
            return clockMin === state.GPU_min_clock && clockMax === state.GPU_max_clock ? state : clone(state);
        }
        case "GPUFreqToggle": {
            if (payload) {
                const clock_min_limits = state.LIMITS_all.gpu.clock_min_limits;
                const clock_max_limits = state.LIMITS_all.gpu.clock_max_limits;
                if (clock_min_limits !== null) {
                    state.GPU_min_clock = clock_min_limits.min;
                }
                if (clock_max_limits !== null) {
                    state.GPU_max_clock = clock_max_limits.max;
                }
            } else {
                state.GPU_min_clock = null;
                state.GPU_max_clock = null;
                await call_backend("GPU_unset_clock_limits", []);
            }
            const clockMin = state.GPU_min_clock;
            const clockMax = state.GPU_max_clock;
            return clockMin === state.GPU_min_clock && clockMax === state.GPU_max_clock ? state : clone(state);
        }
        case "GPUFastPPT": {
            const pptNow = state.GPU_fastPPT;
            const realPpt = payload;
            if (realPpt !== pptNow) {
                const limits: number[] = await call_backend("GPU_set_ppt", [realPpt, state.GPU_slowPPT]); // -> [fastPPT, slowPPT]
                state.GPU_fastPPT = limits[0];
                state.GPU_slowPPT = limits[1];
            }
            const fastPpt = state.GPU_fastPPT;
            const slowPpt = state.GPU_slowPPT;
            return fastPpt === state.GPU_fastPPT && slowPpt === state.GPU_slowPPT ? state : clone(state);
        }
        case "GPUSlowPPT": {
            const pptNow = state.GPU_slowPPT;
            const realPpt = payload;
            // const realPpt = ppt;
            // if (realPpt != pptNow) {
            //   backend.resolve(backend.setGpuPpt(get_value(FAST_PPT_GPU), realPpt),
            if (realPpt !== pptNow) {
                // call_backend("FAST_PPT_GPU",[])
                const limits: number[] = await call_backend("GPU_set_ppt", [state.GPU_fastPPT, payload]); // -> [fastPPT, slowPPT]
                state.GPU_fastPPT = limits[0];
                state.GPU_slowPPT = limits[1];
            }
            const fastPpt = state.GPU_fastPPT;
            const slowPpt = state.GPU_slowPPT;
            return fastPpt === state.GPU_fastPPT && slowPpt === state.GPU_slowPPT ? state : clone(state);
        }
        case "GPUUnsetPPT":
            // TODO
            return state;
        case "GPUPPTToggle": {
            if (state.LIMITS_all.gpu.slow_ppt_limits !== null) {
                state.GPU_slowPPT = state.LIMITS_all.gpu.slow_ppt_limits.max;
            }

            if (state.LIMITS_all.gpu.fast_ppt_limits !== null) {
                state.GPU_fastPPT = state.LIMITS_all.gpu.fast_ppt_limits.max;
            }
            state.GPU_slowPPT = payload;
            state.GPU_fastPPT = payload;
            await call_backend("GPU_unset_ppt", []);
            const fastPpt = state.GPU_fastPPT;
            const slowPpt = state.GPU_slowPPT;
            return fastPpt === state.GPU_fastPPT && slowPpt === state.GPU_slowPPT ? state : clone(state);
        }
        default:
            throw new Error(`Unhandled GPU action ${type}`);
    }
}

export const useGpuReducer = () => useAsyncReducer(reducer, getInitialState);
