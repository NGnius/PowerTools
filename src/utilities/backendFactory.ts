import { getValue, setValue, BackendTypes, ALL } from "../usdpl";
import { assertRequired } from "./assertRequired";

export const Copy = Symbol("copy symbol");

type Copyable<T> = T & Record<typeof Copy, () => Copyable<T>>;

export function backendFactory<
    K extends string,
    T extends { [Key in Extract<K, keyof BackendTypes>]: BackendTypes[Key] }
>(backendDefaults: T): Copyable<T> {
    const obj: Partial<Copyable<T>> = {};

    // using a Symbol for this function property prevents property name collisions without restricting possible property names
    Object.defineProperty(obj, Copy, {
        enumerable: false,
        get() {
            return Object.create(Object.getPrototypeOf(this), Object.getOwnPropertyDescriptors(this));
        },
        set() {
            return false;
        },
    });

    for (const kv of Object.entries(backendDefaults)) {
        const key = kv[0] as keyof BackendTypes;
        const value = kv[1];
        if (key in ALL)
            Object.defineProperty(obj, key, {
                enumerable: true,
                get: () => getValue(key) ?? value,
                set: (newValue: BackendTypes[typeof key]) => setValue(key, newValue),
            });
        else {
            obj[key] = value;
        }
    }
    // use type guard function instead of casting obj type
    assertRequired(obj, Object.keys(backendDefaults) as (keyof T)[]);
    return obj;
}
