import { getValue, setValue, BackendFrameworkMap } from "../usdplFront";
import { assertRequired } from "./assertRequired";

// allows for shallow clones that keep setter/getter property descriptors.
// useful for making shallow equals tests (such as the kind commonly used for
// react state/reducer management) fail
export function clone<T>(obj: T): T {
    return Object.create(Object.getPrototypeOf(obj), Object.getOwnPropertyDescriptors(obj));
}

type BackendObject<T extends keyof BackendFrameworkMap> = { -readonly [K in T]: BackendFrameworkMap[K] };

// this might be better off as a class? or a proxy with a state property that enforces immutability on the state
// property? maybe something with a Map interface (set, get, has) that allows for more flexibility in the future instead
// of marrying BackendObject to object accessor syntax
export function backendFactory<T extends ReadonlyArray<keyof BackendFrameworkMap>>(
    backendKeys: T
): BackendObject<T[number]> {
    const obj: Partial<BackendObject<T[number]>> = {};
    for (const beKey of backendKeys) {
        Object.defineProperty(obj, beKey, {
            enumerable: true,
            get: () => getValue(beKey),
            set: (newValue: BackendFrameworkMap[typeof beKey]) => setValue(beKey, newValue),
        });
    }
    // use type guard function instead of casting obj type
    assertRequired(obj, backendKeys);
    return obj;
}
