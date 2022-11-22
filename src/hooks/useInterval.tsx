import { useEffect } from "react";

export function useInterval(callback: () => void | Promise<void>, ms: number) {
    useEffect(() => {
        function intervalCallback() {
            callback();
        }
        const intervalHandle = setInterval(intervalCallback, ms);
        return () => clearInterval(intervalHandle);
    }, [ms]);
}
