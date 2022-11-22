import { call_backend, get_value, set_value } from "./augmentedUsdplFront";
import { reload } from "./reload";

export const periodicals = async function (usdplReady: boolean, smtAllowedRef: { current: boolean }) {
    await call_backend("GENERAL_get_name", [])
        .then(([newName]) => {
            const prevName = get_value("GENERAL_name");
            set_value("GENERAL_name", newName);
            return newName !== prevName ? reload(usdplReady, smtAllowedRef) : Promise.resolve();
        })
        .then(() => void "reload complete"),
    await Promise.all([
        call_backend("BATTERY_current_now", []).then(([rate]) => set_value("BATTERY_current_now", rate)),
        call_backend("BATTERY_charge_now", []).then(([rate]) => set_value("BATTERY_charge_now", rate)),
        call_backend("BATTERY_charge_full", []).then(([rate]) => set_value("BATTERY_charge_full", rate)),
        call_backend("GENERAL_get_persistent", []).then(([value]) => set_value("GENERAL_persistent", value)),
    ]);
};
