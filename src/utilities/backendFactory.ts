import { BackendTypes, BackendObject, getValue, setValue } from "../usdplFront";
import { assertRequired } from "./assertRequired";

/** symbol `Copy` is used as the property name for a `backendFactory` object's self-copying function */
export const Copy = Symbol("backendFactoryCopy");

type Copyable<T> = T & Record<typeof Copy, () => Copyable<T>>;

function makeObjectSelfCopying<T>(arg: T): Copyable<T> {
    // use a symbol to prevent collisions with keys from `arg` source object
    return Object.defineProperty(arg, Copy, {
        enumerable: false,
        get() {
            return Object.create(Object.getPrototypeOf(this), Object.getOwnPropertyDescriptors(this));
        },
        set() {
            return false;
        },
    }) as Copyable<T>;
}

export function backendFactory<T extends keyof BackendTypes>(keys: T[]): Copyable<BackendObject<T>> {
    const backendObject: Partial<BackendObject<T>> = {};
    for (const key of keys) {
        Object.defineProperty(backendObject, key, {
            enumerable: true,
            get: () => getValue(key),
            set: (newValue: BackendTypes[typeof key]) => setValue(key, newValue),
        });
    }
    // use type guard function instead of casting obj type
    assertRequired(backendObject, keys);

    return makeObjectSelfCopying(backendObject);
}
