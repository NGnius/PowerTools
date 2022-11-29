import { BACKEND_CALLS, callBackend, GeneralTypes } from "../usdpl";
import { useAsyncReducer } from "../hooks/useAsyncReducer";
import { Copy } from "../utilities/backendFactory";
import { GENERAL_BE } from "../usdplBackend";

type Action =
    | [type: "loadSystemDefaults", payload: () => Promise<void>]
    | [type: "setPersistent", payload: boolean]
    | [type: "idk"]
    | [type: "refresh"];

async function reducer(state: GeneralTypes, action: Action) {
    const [type, payload] = action;

    console.debug(`General Action: ${type}; Payload: ${payload}`);

    switch (type) {
        case "idk":
            callBackend(BACKEND_CALLS.GeneralIdk, []);
            return state;
        case "setPersistent": {
            const [newValue] = await callBackend(BACKEND_CALLS.GeneralSetPersistent, [payload]);
            state.GENERAL_persistent = newValue;
            return state[Copy]();
        }
        case "refresh":
            return state[Copy]();
        case "loadSystemDefaults": {
            const [newValue] = await callBackend(BACKEND_CALLS.GeneralSetPersistent, [false]);
            state.GENERAL_persistent = newValue;
            await callBackend(BACKEND_CALLS.GeneralLoadSystemSettings, []);
            await payload();
            await callBackend(BACKEND_CALLS.GeneralWaitForUnlocks, []);
            return state[Copy]();
        }
        default:
            throw new Error(`Unhandled General Action ${type}`);
    }
}

export const useGeneralReducer = () => useAsyncReducer(reducer, () => GENERAL_BE);
