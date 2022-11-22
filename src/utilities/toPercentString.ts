export const toPercentString = (a: unknown, b: unknown, unit: string): string =>
    typeof a === "number" && typeof b === "number" && b !== 0
        ? `${a.toFixed(1)} ${unit} (${((100 * a) / b).toFixed(1)}%)`
        : "";
