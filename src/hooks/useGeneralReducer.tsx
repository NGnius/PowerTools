import { useAsyncReducer } from "../hooks/useAsyncReducer";
import { call_backend } from "../utilities/augmentedUsdplFront";
import { backendFactory, clone } from "../utilities/backendFactory";
import { reload } from "../utilities/reload";

type Action =
    | [type: "LoadSystemDefaults", payload?: undefined]
    | [type: "SetPersistent", payload: boolean]
    | [type: "refresh"];

const getInitialState = () => backendFactory(["V_INFO", "GENERAL_persistent", "GENERAL_name"]);

type State = ReturnType<typeof getInitialState>;

async function reducer(state: State, action: Action) {
    const [type, payload] = action;
    console.debug(`General Action: ${type}; Payload: ${payload}`);
    switch (type) {
        case "SetPersistent": {
            const [newValue] = await call_backend("GENERAL_set_persistent", [payload]);
            state.GENERAL_persistent = newValue;
            return clone(state);
        }
        case "refresh":
            return clone(state);
        case "LoadSystemDefaults": {
            const [newValue] = await call_backend("GENERAL_set_persistent", [false]);
            state.GENERAL_persistent = newValue;
            await call_backend("GENERAL_load_system_settings", []);
            // TODO - pass values from react -- set context that imports
            // module-scoped vars, which can be imported in all JS regardless of
            // framework?
            await reload(true, { current: true });
            await call_backend("GENERAL_wait_for_unlocks", []);
            return clone(state);
        }
        default:
            throw new Error(`Unhandled General Action ${type}`);
    }
}

export const useGeneralReducer = () => useAsyncReducer(reducer, getInitialState);
