export function assertRequired<T>(obj: Partial<T>, props: ReadonlyArray<keyof T>): asserts obj is T;
export function assertRequired<T>(obj: Partial<T>, props: Record<keyof T, unknown>): asserts obj is T;
export function assertRequired<T>(
    obj: Partial<T>,
    props: Record<keyof T, unknown> | ReadonlyArray<keyof T>
): asserts obj is T {
    if (Array.isArray(props)) {
        for (const key of props) {
            if (!(key in obj)) {
                throw new TypeError(`Object does not have the required properties ${String(key)} of ${props}`);
            }
        }
    } else {
        for (const key in props) {
            if (!(key in obj)) {
                throw new TypeError(`Object does not have the required properties ${String(key)} of ${props}`);
            }
        }
    }
}
