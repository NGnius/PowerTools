import { get_value, set_value, BackendProperyMap } from "../utilities/augmentedUsdplFront";

// allows for shallow clones that keep setter/getter property descriptors.
// useful for making shallow equals tests (such as the kind commonly used for
// react state/reducer management) fail
export function clone<T>(obj: T): T {
    return Object.create(Object.getPrototypeOf(obj), Object.getOwnPropertyDescriptors(obj));
}

export type BackendObject<T extends keyof BackendProperyMap> = { -readonly [K in T]: BackendProperyMap[K] };

// this might be better off as a class?
export function backendFactory<T extends (keyof BackendProperyMap)[]>(backendKeys: T): BackendObject<T[number]> {
    const obj = {};
    for (const beKey of backendKeys) {
        Object.defineProperty(obj, beKey, {
            enumerable: true,
            get: () => get_value(beKey),
            set: (newValue: BackendProperyMap[typeof beKey]) => set_value(beKey, newValue),
        });
    }
    return Object.seal(obj) as BackendObject<T[number]>;
}
