import { definePlugin, ServerAPI, staticClasses } from "decky-frontend-lib";
import { useEffect, useState, VFC } from "react";
import { GiDrill } from "react-icons/gi";
import { initialize } from "./initialize";
import { Content } from "./Content";
import { ErrorBoundary } from "./ErrorBoundary";
interface Registerable {
    unregister?: () => void;
}
export type GlobalRefs = {
    lifetimeHook: Registerable | null;
    startHook: Registerable | null;
    usdplReady: boolean;
};
const globalRefs: GlobalRefs = {
    lifetimeHook: null,
    startHook: null,
    usdplReady: false,
};
const App: VFC<{ serverAPI: ServerAPI }> = ({ serverAPI }) => {
    const [isInitializing, setIsInitializing] = useState<boolean | "timeout">(false);
    useEffect(() => {
        setIsInitializing(true);
        const handleTimeout = () => setIsInitializing("timeout");
        const timeoutHandle = setTimeout(handleTimeout, 3000);
        initialize(globalRefs).then(() => {
            clearTimeout(timeoutHandle);
            setIsInitializing(false);
        });
    }, []);

    return isInitializing === true ? (
        <>PowerTools is starting up...</>
    ) : (
        <ErrorBoundary>
            {isInitializing === "timeout" && <h3>Timeout!</h3>}
            <Content serverAPI={serverAPI} globalRefs={globalRefs} />
        </ErrorBoundary>
    );
};

export default definePlugin((serverAPI: ServerAPI) => ({
    title: <div className={staticClasses.Title}>PowerTools</div>,
    content: (
        <ErrorBoundary>
            <App serverAPI={serverAPI} />
        </ErrorBoundary>
    ),
    icon: <GiDrill />,
    onDismount() {
        console.debug("PowerTools shutting down");
        globalRefs.lifetimeHook?.unregister?.();
        globalRefs.startHook?.unregister?.();
        serverAPI.routerHook.removeRoute("/decky-plugin-test");
        console.debug("Unregistered PowerTools callbacks, goodbye.");
    },
}));
