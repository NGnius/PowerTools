import { useCallback, useRef, useState } from "react";

type Primitive = string | boolean | number | undefined | null | symbol;

// eslint-disable-next-line @typescript-eslint/ban-types
type ExcludeFnProps<T> = Pick<T, { [K in keyof T]: T[K] extends Function ? never : K }[keyof T]>;

// prevent eslint from forcing from interface -> type
// eslint-disable-next-line
interface Json extends ExcludeFnProps<{ [K in string]: Primitive | Json | Primitive[] | Json[] } | Primitive[]> {}

type AsyncReducer<S, A> = (prevState: S, action: A) => Promise<S>;

export type AsyncDispatch<A> = (action: A) => Promise<void>;

type UseAsyncReducer = <S extends Json, A>(
    reducer: AsyncReducer<S, A>,
    getInitialState: () => S
) => [state: Readonly<S>, dispatch: AsyncDispatch<A>, refetchState: () => void];

export const useAsyncReducer: UseAsyncReducer = function useAsyncReducer(reducer, getInitialState) {
    const [state, setState] = useState(getInitialState());

    const stateRef = useRef(state);
    stateRef.current = state;

    const getInitialStateRef = useRef(getInitialState);
    getInitialStateRef.current = getInitialState;

    const asyncDispatch = useCallback(
        async (action: Parameters<typeof reducer>[1]) => {
            const updatedState = await reducer(stateRef.current, action);
            // shallow equality check -- reducer must ensure a new obj is returned if state changes
            if (stateRef.current !== updatedState) {
                setState(updatedState);
            }
        },
        [reducer]
    );
    const refetchState = useCallback(() => {
        setState((prev) => ({ ...prev, ...getInitialStateRef.current() }));
    }, []);

    return [state, asyncDispatch, refetchState];
};
