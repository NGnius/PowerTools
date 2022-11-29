const CORES = 8;

export function fill<T>(arg: T, length = CORES): T[] {
    return new Array(length).fill(Object.freeze(arg));
}

export function notNull<T>(arg: T | null | undefined): arg is T {
    return arg !== null && typeof arg !== "undefined";
}
export function isNull<T>(arg: T | null | undefined): arg is null | undefined {
    return arg !== null && typeof arg !== "undefined";
}

export function isNumber<K extends string>(
    arg: Partial<Record<string, unknown>> | null | undefined,
    key: K
): arg is (Omit<typeof arg, K> & Record<K, number>) & typeof arg {
    return !!arg && key in arg && typeof arg[key] === "number";
}

export const toPercentString = (a: unknown, b: unknown, unit: string): string =>
    typeof a === "number" && typeof b === "number" && b !== 0
        ? `${a.toFixed(1)} ${unit} (${((100 * a) / b).toFixed(1)}%)`
        : "";

export function countCpus(statii: boolean[]): number {
    let count = 0;
    for (let i = 0; i < statii.length; i++) {
        if (statii[i]) {
            count += 1;
        }
    }
    return count;
}
