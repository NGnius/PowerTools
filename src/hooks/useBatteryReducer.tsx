import { useAsyncReducer } from "./useAsyncReducer";
import { call_backend } from "../utilities/augmentedUsdplFront";
import { backendFactory, clone } from "../utilities/backendFactory";

const getInitialState = () => backendFactory([
    "BATTERY_current_now",
    "BATTERY_charge_rate",
    "BATTERY_charge_now",
    "BATTERY_charge_full",
    "BATTERY_charge_design",
    "BATTERY_charge_mode",
]);

type State = ReturnType<typeof getInitialState>;

type Action =
    | [type: "BATTChargeRateToggle", payload: boolean]
    | [type: "BATTUnsetChargeRate", payload?: undefined]
    | [type: "BATTChargeRate", payload: number]
    | [type: "BATTChargeMode", payload: string]
    | [type: "BATTChargeModeToggle", payload: { toggle: boolean; value: string }]
    | [type: "refresh"];

async function reducer(state: State, action: Action) {
    const [type, payload] = action;
    console.debug(`Battery Action: ${type}; Payload: ${payload}`);
    switch (type) {
        case "refresh": {
            await Promise.all([
                call_backend("BATTERY_current_now", []).then(([rate]) => (state.BATTERY_current_now = rate)),
                call_backend("BATTERY_charge_now", []).then(([rate]) => (state.BATTERY_charge_now = rate)),
                call_backend("BATTERY_charge_full", []).then(([rate]) => (state.BATTERY_charge_full = rate)),
            ]);
            return clone(state);
        }
        case "BATTChargeModeToggle": {
            if (payload.toggle) {
                // no backend call?
                state.BATTERY_charge_mode = payload.value;
            } else {
                state.BATTERY_charge_mode = null;
                await call_backend("BATTERY_unset_charge_mode", []);
            }
            return clone(state);
        }
        case "BATTChargeMode": {
            console.debug("Charge mode dropdown selected", payload);
            const [mode] = await call_backend("BATTERY_set_charge_mode", [payload]);
            state.BATTERY_charge_mode = mode;
            return clone(state);
        }
        case "BATTChargeRate": {
            const prevRate = state.BATTERY_charge_rate;
            const rateNow = state.BATTERY_charge_rate;
            if (payload !== rateNow) {
                const [rate] = await call_backend("BATTERY_set_charge_rate", [payload]);
                state.BATTERY_charge_rate = rate;
            }
            return prevRate === state.BATTERY_charge_rate ? state : clone(state);
        }
        case "BATTChargeRateToggle": {
            if (payload) {
                state.BATTERY_charge_rate = 2500;
            } else {
                state.BATTERY_charge_rate = null;
                await call_backend("BATTERY_unset_charge_rate", []);
            }
            return clone(state);
        }
        default:
            throw new Error(`Unhandled Battery action ${type}`);
    }
}

export const useBatteryReducer = () => useAsyncReducer(reducer, getInitialState);
