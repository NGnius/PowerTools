/* eslint-disable @typescript-eslint/ban-ts-comment */
import { init_embedded, init_usdpl, target_usdpl } from "usdpl-front";
import { setValue, GENERAL, callBackend, BACKEND_CALLS } from "./usdpl";

const USDPL_PORT = 44443;

// init USDPL WASM and connection to back-end
export async function initialize(refs: {
    lifetimeHook: unknown | null;
    startHook: unknown | null;
    usdplReady: boolean;
}) {
    await init_embedded();
    init_usdpl(USDPL_PORT);
    console.log("USDPL started for framework: " + target_usdpl());
    refs.usdplReady = true;
    setValue(GENERAL.Name, "Default");

    // register Steam callbacks
    refs.lifetimeHook = SteamClient.GameSessions.RegisterForAppLifetimeNotifications(
        (update: { bRunning: unknown }) => {
            if (update.bRunning) {
                //console.debug("AppID " + update.unAppID.toString() + " is now running");
            } else {
                //console.debug("AppID " + update.unAppID.toString() + " is no longer running");
                callBackend(BACKEND_CALLS.GeneralLoadDefaultSettings, []).then(([ok]) =>
                    console.debug("Loading default settings ok? " + ok)
                );
            }
        }
    );
    refs.startHook = SteamClient.Apps.RegisterForGameActionStart((_: unknown, id: { toString: () => string }) => {
        // @ts-ignore
        const gameInfo = appStore.GetAppOverviewByGameID(id);
        // don't use gameInfo.appid, haha
        callBackend(BACKEND_CALLS.GeneralLoadSettings, [id.toString() + ".json", gameInfo.display_name]).then(([ok]) =>
            console.debug("Loading settings ok? " + ok)
        );
    });

    console.debug("Registered PowerTools callbacks, hello!");
}
