import { useEffect, useMemo, useRef } from "react";

export function useInterval0(callback: () => void | Promise<void>, ms: number) {
    const cbRef = useRef(callback);
    cbRef.current = callback;
    useEffect(() => {
        function intervalCallback() {
            cbRef.current();
        }
        const intervalHandle = setInterval(intervalCallback, ms);
        return () => clearInterval(intervalHandle);
    }, [ms]);
}

type UseInterval = (callback: () => void | Promise<void>, deps: unknown[]) => void;

export const intervalHookFactory =
    (ms: number): UseInterval =>
        (callback: () => void | Promise<void>, deps: unknown[]) => {
        // eslint-disable-next-line react-hooks/exhaustive-deps
            const memoizedCb = useMemo(() => callback, deps);

            useEffect(() => {
                function intervalCallback() {
                    memoizedCb();
                }
                const intervalHandle = setInterval(intervalCallback, ms);
                return () => clearInterval(intervalHandle);
            }, [memoizedCb]);
        };
