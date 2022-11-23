import { General, BackendCalls, callBackend } from "../usdplFront";
import { useAsyncReducer } from "../hooks/useAsyncReducer";
import { backendFactory, clone } from "../utilities/backendFactory";

type Action =
    | [type: "loadSystemDefaults", payload: () => Promise<void>]
    | [type: "setPersistent", payload: boolean]
    | [type: "idk"]
    | [type: "refresh"];

const getInitialState = () => backendFactory([General.VInfo, General.Persistent, General.Name]);

type State = ReturnType<typeof getInitialState>;

async function reducer(state: State, action: Action) {
    const [type, payload] = action;

    console.debug(`General Action: ${type}; Payload: ${payload}`);

    switch (type) {
        case "idk":
            callBackend(BackendCalls.GeneralIdk, []);
            return state;
        case "setPersistent": {
            const [newValue] = await callBackend(BackendCalls.GeneralSetPersistent, [payload]);
            state.GENERAL_persistent = newValue;
            return clone(state);
        }
        case "refresh":
            return clone(state);
        case "loadSystemDefaults": {
            const [newValue] = await callBackend(BackendCalls.GeneralSetPersistent, [false]);
            state.GENERAL_persistent = newValue;
            await callBackend(BackendCalls.GeneralLoadSystemSettings, []);
            await payload();
            await callBackend(BackendCalls.GeneralWaitForUnlocks, []);
            return clone(state);
        }
        default:
            throw new Error(`Unhandled General Action ${type}`);
    }
}

export const useGeneralReducer = () => useAsyncReducer(reducer, getInitialState);
