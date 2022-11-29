export function assertRequired<T>(obj: Partial<T>, props: ReadonlyArray<keyof T>): asserts obj is T {
    for (const key of props) {
        if (!(key in obj)) {
            throw new TypeError(`Object does not have the required properties ${String(key)} of ${props}`);
        }
    }
}
