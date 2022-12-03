import { BACKEND_CALLS, BATTERY, BatteryTypes, callBackend } from "../usdplFront";
import { backendFactory, Copy } from "../utilities/backendFactory";
import { useAsyncReducer } from "./useAsyncReducer";

type Action =
    | [type: "chargeRateToggle", payload: boolean]
    | [type: "unsetChargeRate", payload?: undefined]
    | [type: "chargeRate", payload: number]
    | [type: "chargeMode", payload: string]
    | [type: "chargeModeToggle", payload: { toggle: boolean; value: string }]
    | [type: "refresh"];

async function reducer(state: Partial<BatteryTypes>, action: Action): Promise<typeof state> {
    const [type, payload] = action;

    console.debug(`Battery Action: ${type}; Payload: ${payload}`);

    switch (type) {
        case "refresh": {
            await Promise.all([
                callBackend(BACKEND_CALLS.BatteryCurrentNow, []).then(([data]) => (state.BATTERY_current_now = data)),
                callBackend(BACKEND_CALLS.BatteryChargeNow, []).then(([data]) => (state.BATTERY_charge_now = data)),
                callBackend(BACKEND_CALLS.BatteryChargeFull, []).then(([data]) => (state.BATTERY_charge_full = data)),
            ]);
            return state[Copy]();
        }
        case "chargeModeToggle": {
            if (payload.toggle) {
                state.BATTERY_charge_mode = payload.value;
            } else {
                state.BATTERY_charge_mode = null;
                await callBackend(BACKEND_CALLS.BatteryUnsetChargeMode, []);
            }
            return state[Copy]();
        }
        case "chargeMode": {
            console.debug("Charge mode dropdown selected", payload);
            const [mode] = await callBackend(BACKEND_CALLS.BatterySetChargeMode, [payload]);
            state.BATTERY_charge_mode = mode;
            return state[Copy]();
        }
        case "chargeRate": {
            const prevRate = state.BATTERY_charge_rate;
            const rateNow = state.BATTERY_charge_rate;
            if (payload !== rateNow) {
                const [rate] = await callBackend(BACKEND_CALLS.BatterySetChargeRate, [payload]);
                state.BATTERY_charge_rate = rate;
            }
            return prevRate === state.BATTERY_charge_rate ? state : state[Copy]();
        }
        case "chargeRateToggle": {
            if (payload) {
                state.BATTERY_charge_rate = 2500;
            } else {
                state.BATTERY_charge_rate = null;
                await callBackend(BACKEND_CALLS.BatteryUnsetChargeRate, []);
            }
            return state[Copy]();
        }
        default:
            throw new Error(`Unhandled Battery action ${type}`);
    }
}

export const useBatteryReducer = () =>
    useAsyncReducer(reducer, () => backendFactory(Object.values(BATTERY) as (keyof BatteryTypes)[]));
