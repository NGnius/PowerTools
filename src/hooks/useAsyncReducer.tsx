import { useCallback, useState } from "react";

type Primitive = string | boolean | number | undefined | null | symbol;
type FunctionShape = ((...arg: unknown[]) => unknown) | (() => unknown);
type ExcludeFunctionProperties<T> = Pick<T, { [K in keyof T]: T[K] extends FunctionShape ? never : K }[keyof T]>;

// prevent eslint from forcing from interface -> type
// eslint-disable-next-line
interface Json
    extends ExcludeFunctionProperties<{ [K in string]: Primitive | Json | Primitive[] | Json[] } | Primitive[]> {}

type AsyncReducer<S, A> = (prevState: S, action: A) => Promise<S>;

type UseAsyncReducer = <S extends Json, A>(
    reducer: AsyncReducer<S, A>,
    getInitialState: () => S
) => [state: Readonly<S>, dispatch: (action: A) => Promise<void>];

export const useAsyncReducer: UseAsyncReducer = function useAsyncReducer(reducer, getInitialState) {
    const [state, setState] = useState(getInitialState());
    const asyncDispatch = useCallback(
        async function (action) {
            const updatedState = await reducer(state, action);
            if (updatedState !== state) {
                setState(updatedState);
            }
        },
        [state, reducer]
    );
    return [state, asyncDispatch];
};
