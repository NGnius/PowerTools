import "decky-frontend-lib";
import { SteamClient as DeckySteamClient } from "decky-frontend-lib";
declare module "*.svg" {
    const content: string;
    export default content;
}

declare module "*.png" {
    const content: string;
    export default content;
}

declare module "*.jpg" {
    const content: string;
    export default content;
}

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

export interface MinMax {
    min: number | null;
    max: number | null;
}
