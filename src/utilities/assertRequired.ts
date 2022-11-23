const hasOwn = (obj: unknown, prop: string | number | symbol) => Object.prototype.hasOwnProperty.call(obj, prop);

export function assertRequired<T>(obj: Partial<T>, props: ReadonlyArray<keyof T>): asserts obj is T {
    if (!props.every((key) => hasOwn(obj, key)))
        throw new TypeError(`Object does not have the required properties ${props}`);
}
