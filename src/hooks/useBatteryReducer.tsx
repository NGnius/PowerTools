import { callBackend, Battery, BackendCalls } from "../usdplFront";
import { backendFactory, clone } from "../utilities/backendFactory";
import { useAsyncReducer } from "./useAsyncReducer";

const getInitialState = () =>
    backendFactory([
        Battery.CurrentNow,
        Battery.ChargeRate,
        Battery.ChargeNow,
        Battery.ChargeFull,
        Battery.ChargeDesign,
        Battery.ChargeMode,
    ]);

type State = ReturnType<typeof getInitialState>;

type Action =
    | [type: "chargeRateToggle", payload: boolean]
    | [type: "unsetChargeRate", payload?: undefined]
    | [type: "chargeRate", payload: number]
    | [type: "chargeMode", payload: string]
    | [type: "chargeModeToggle", payload: { toggle: boolean; value: string }]
    | [type: "refresh"];

async function reducer(state: State, action: Action) {
    const [type, payload] = action;

    console.debug(`Battery Action: ${type}; Payload: ${payload}`);

    switch (type) {
        case "refresh": {
            await Promise.all([
                callBackend(BackendCalls.BatteryCurrentNow, []).then(([data]) => (state.BATTERY_current_now = data)),
                callBackend(BackendCalls.BatteryChargeNow, []).then(([data]) => (state.BATTERY_charge_now = data)),
                callBackend(BackendCalls.BatteryChargeFull, []).then(([data]) => (state.BATTERY_charge_full = data)),
            ]);
            return clone(state);
        }
        case "chargeModeToggle": {
            if (payload.toggle) {
                state.BATTERY_charge_mode = payload.value;
            } else {
                state.BATTERY_charge_mode = null;
                await callBackend(BackendCalls.BatteryUnsetChargeMode, []);
            }
            return clone(state);
        }
        case "chargeMode": {
            console.debug("Charge mode dropdown selected", payload);
            const [mode] = await callBackend(BackendCalls.BatterySetChargeMode, [payload]);
            state.BATTERY_charge_mode = mode;
            return clone(state);
        }
        case "chargeRate": {
            const prevRate = state.BATTERY_charge_rate;
            const rateNow = state.BATTERY_charge_rate;
            if (payload !== rateNow) {
                const [rate] = await callBackend(BackendCalls.BatterySetChargeRate, [payload]);
                state.BATTERY_charge_rate = rate;
            }
            return prevRate === state.BATTERY_charge_rate ? state : clone(state);
        }
        case "chargeRateToggle": {
            if (payload) {
                state.BATTERY_charge_rate = 2500;
            } else {
                state.BATTERY_charge_rate = null;
                await callBackend(BackendCalls.BatteryUnsetChargeRate, []);
            }
            return clone(state);
        }
        default:
            throw new Error(`Unhandled Battery action ${type}`);
    }
}

export const useBatteryReducer = () => useAsyncReducer(reducer, getInitialState);
