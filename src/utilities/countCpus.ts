export function countCpus(statii: boolean[]): number {
    let count = 0;
    for (let i = 0; i < statii.length; i++) {
        if (statii[i]) {
            count += 1;
        }
    }
    return count;
}
