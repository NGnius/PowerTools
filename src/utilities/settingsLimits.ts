import { limits } from "../usdplBackend";
import { GENERAL, getValue, SettingsLimits } from "../usdpl";
import { assertRequired } from "./assertRequired";

const settingsLimits: Partial<SettingsLimits> = {};

const props: (keyof SettingsLimits)[] = ["cpu", "gpu", "general", "battery"];

for (const key of props) {
    Object.defineProperty(settingsLimits, key, {
        enumerable: true,
        get: () => getValue(GENERAL.LimitsAll)?.[key] ?? limits?.[key],
        // eslint-disable-next-line @typescript-eslint/no-empty-function
        set: (): void => {},
    });
}

assertRequired(settingsLimits, props);

export const SETTINGS_LIMITS = settingsLimits;
