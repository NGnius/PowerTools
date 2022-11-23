import { SteamClient as DeckySteamClient } from "decky-frontend-lib";

// TODO: make PR against decky-frontend-lib with these types

interface Apps extends DeckySteamClient["Apps"] {
    RegisterForGameActionStart: (callback: (actionType: unknown, id: number) => void) => Registerable;
}

interface GameSessions extends DeckySteamClient["GameSessions"] {
    RegisterForAppLifetimeNotifications: (callback: (update: { bRunning: boolean }) => void) => Registerable;
}
declare global {
    interface Registerable {
        unregister?: () => void;
    }

    interface SteamClient extends DeckySteamClient {
        GameSessions: GameSessions;
        Apps: Apps;
    }
    const appStore: {
        GetAppOverviewByGameID: (id: unknown) => { display_name: string };
    };
}
