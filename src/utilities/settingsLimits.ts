import { GENERAL, getValue, LIMITS, SettingsLimits } from "../usdplFront";
import { assertRequired } from "./assertRequired";

const settingsLimits: Partial<SettingsLimits> = {};

const props: (keyof SettingsLimits)[] = ["cpu", "gpu", "general", "battery"];

for (const key of props) {
    Object.defineProperty(settingsLimits, key, {
        enumerable: true,
        get: () => getValue(GENERAL.LimitsAll)?.[key] ?? LIMITS?.[key],
        // eslint-disable-next-line @typescript-eslint/no-empty-function
        set: (): void => {},
    });
}

assertRequired(settingsLimits, props);

export const SETTINGS_LIMITS = settingsLimits;
