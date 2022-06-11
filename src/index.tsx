import {
  //ButtonItem,
  definePlugin,
  DialogButton,
  //Menu,
  //MenuItem,
  PanelSection,
  PanelSectionRow,
  //Router,
  ServerAPI,
  //showContextMenu,
  staticClasses,
  Slider,
  Toggle,
  //NotchLabel
  gamepadDialogClasses,
  joinClassNames,
} from "decky-frontend-lib";
import { VFC, useState } from "react";
import { GiDrill } from "react-icons/gi";

import * as python from "./python";

//import logo from "../assets/logo.png";

// interface AddMethodArgs {
//   left: number;
//   right: number;
// }

var firstTime: boolean = true;
var versionGlobal: string = "0.0.0-jank";
var periodicHook: NodeJS.Timer | null = null;
var lastGame: string = "";
var lifetimeHook: any = null;
var startHook: any = null;

var reload = function(){};

const Content: VFC<{ serverAPI: ServerAPI }> = ({serverAPI}) => {
  // const [result, setResult] = useState<number | undefined>();

  // const onClick = async () => {
  //   const result = await serverAPI.callPluginMethod<AddMethodArgs, number>(
  //     "add",
  //     {
  //       left: 2,
  //       right: 2,
  //     }
  //   );
  //   if (result.success) {
  //     setResult(result.result);
  //   }
  // };

  python.setServer(serverAPI);

  const [smtGlobal, setSMT] = useState<boolean>(true);
  const [cpusGlobal, setCPUs] = useState<number>(8);
  const [boostGlobal, setBoost] = useState<boolean>(true);

  const [freqGlobal, setFreq] = useState<number>(8);

  const [slowPPTGlobal, setSlowPPT] = useState<number>(1);
  const [fastPPTGlobal, setFastPPT] = useState<number>(1);

  const [chargeNowGlobal, setChargeNow] = useState<number>(40);
  const [chargeFullGlobal, setChargeFull] = useState<number>(40);
  const [chargeDesignGlobal, setChargeDesign] = useState<number>(40);

  const [persistGlobal, setPersist] = useState<boolean>(false);
  const [perGameProfileGlobal, setPerGameProfile] = useState<boolean>(false);
  const [gameGlobal, setGame] = useState<string>("with your mom");

  reload = function () {
      python.execute(python.onViewReady());

      python.resolve(python.getSMT(), setSMT);
      python.resolve(python.getCPUs(), setCPUs);
      python.resolve(python.getCPUBoost(), setBoost);
      python.resolve(python.getMaxBoost(), setFreq);

      python.resolve(python.getGPUPowerI(1), setSlowPPT);
      python.resolve(python.getGPUPowerI(2), setFastPPT);

      python.resolve(python.getPersistent(), setPersist);
      python.resolve(python.getPerGameProfile(), setPerGameProfile);
    };


  if (firstTime) {
    firstTime = false;

    reload(); // technically it's just load, not reload ;)

    python.resolve(python.getChargeNow(), setChargeNow);
    python.resolve(python.getChargeFull(), setChargeFull);
    python.resolve(python.getChargeDesign(), setChargeDesign);

    python.resolve(python.getCurrentGame(), setGame);

    periodicHook = setInterval(function() {
        python.resolve(python.getChargeNow(), setChargeNow);
        python.resolve(python.getChargeFull(), setChargeFull);
        python.resolve(python.getCurrentGame(), (game: string) => {
          if (lastGame != game) {
            setGame(game);
            lastGame = game;
            reload();
          }
        });
    }, 1000);

    python.resolve(python.getVersion(), (v: string) => {versionGlobal = v;});

    //@ts-ignore
    lifetimeHook = SteamClient.GameSessions.RegisterForAppLifetimeNotifications((update) => {
        if (update.bRunning) {
            console.log("AppID " + update.unAppID.toString() + " is now running");
        } else {
            console.log("AppID " + update.unAppID.toString() + " is no longer running");
            python.execute(python.onGameStop(null));
        }
    });
    //@ts-ignore
    SteamClient.Apps.RegisterForGameActionStart((actionType, id) => {
        //@ts-ignore
        let gameInfo: any = appStore.GetAppOverviewByGameID(id);
        python.execute(python.onGameStart(id, gameInfo));
    });
  }

  const FieldWithSeparator = joinClassNames(gamepadDialogClasses.Field, gamepadDialogClasses.WithBottomSeparatorStandard);

  return (
    <PanelSection>
      {/* CPU */}
      <div className={staticClasses.PanelSectionTitle}>
        CPU
      </div>
      <PanelSectionRow>
        <Toggle
          checked={smtGlobal}
          label="SMT"
          description="Enables odd-numbered CPUs"
          onChange={(smt: boolean) => {
            console.log("SMT is now " + smt.toString());
            python.execute(python.setCPUs(cpusGlobal, smt));
            python.resolve(python.getCPUs(), setCPUs);
            python.resolve(python.getSMT(), setSMT);
          }}
        />
      </PanelSectionRow>
      <PanelSectionRow>
        <Slider
          label="Threads"
          value={cpusGlobal}
          step={1}
          max={smtGlobal? 8 : 4}
          min={1}
          showValue={true}
          onChange={(cpus: number) => {
            console.log("CPU slider is now " + cpus.toString());
            if (cpus != cpusGlobal) {
              python.execute(python.setCPUs(cpus, smtGlobal));
              python.resolve(python.getCPUs(), setCPUs);
            }
          }}
        />
      </PanelSectionRow>
      <PanelSectionRow>
        <Toggle
          checked={boostGlobal}
          label="Boost"
          description="Allows the CPU to go above max frequency"
          onChange={(boost: boolean) => {
            console.log("Boost is now " + boost.toString());
            python.execute(python.setCPUBoost(boost));
            python.resolve(python.getCPUBoost(), setBoost);
          }}
        />
      </PanelSectionRow>
      <PanelSectionRow>
        <Slider
          label="Max Frequency"
          value={freqGlobal}
          max={2}
          min={0}
          notchCount={3}
          notchLabels={[
            {notchIndex: 0, label: "1.7GHz"},
            {notchIndex: 1, label: "2.4GHz"},
            {notchIndex: 2, label: "2.8GHz"},
          ]}
          notchTicksVisible={true}
          onChange={(freq: number) => {
            console.log("CPU slider is now " + freq.toString());
            if (freq != freqGlobal) {
              python.execute(python.setMaxBoost(freq));
              python.resolve(python.getMaxBoost(), setFreq);
            }
          }}
        />
      </PanelSectionRow>
      {/* GPU */}
      <div className={staticClasses.PanelSectionTitle}>
        GPU
      </div>
      <PanelSectionRow>
        {/* index: 1 */}
        <Slider
          label="SlowPPT Power"
          value={slowPPTGlobal}
          max={2}
          min={0}
          notchCount={3}
          notchLabels={[
            {notchIndex: 0, label: "Min"},
            {notchIndex: 1, label: "Auto"},
            {notchIndex: 2, label: "Max"},
          ]}
          notchTicksVisible={true}
          onChange={(ppt: number) => {
            console.log("SlowPPT is now " + ppt.toString());
            if (ppt != slowPPTGlobal) {
              python.execute(python.setGPUPowerI(ppt, 1));
              python.resolve(python.getGPUPowerI(1), setSlowPPT);
            }
          }}
        />
      </PanelSectionRow>
      <PanelSectionRow>
        {/* index: 2 */}
        <Slider
          label="FastPPT Power"
          value={fastPPTGlobal}
          max={2}
          min={0}
          notchCount={3}
          notchLabels={[
            {notchIndex: 0, label: "Min"},
            {notchIndex: 1, label: "Auto"},
            {notchIndex: 2, label: "Max"},
          ]}
          notchTicksVisible={true}
          onChange={(ppt: number) => {
            console.log("FastPPT is now " + ppt.toString());
            if (ppt != fastPPTGlobal) {
              python.execute(python.setGPUPowerI(ppt, 2));
              python.resolve(python.getGPUPowerI(2), setFastPPT);
            }
          }}
        />
      </PanelSectionRow>
      {/* Battery */}
      <div className={staticClasses.PanelSectionTitle}>
        Battery
      </div>
      <PanelSectionRow>
        <div className={FieldWithSeparator}>
          <div className={gamepadDialogClasses.FieldLabelRow}>
            <div className={gamepadDialogClasses.FieldLabel}>
            Now (Charge)
            </div>
            <div className={gamepadDialogClasses.FieldChildren}>
            {(7.7 * chargeNowGlobal / 1000000).toFixed(1).toString() + " Wh (" + (100 * chargeNowGlobal / chargeFullGlobal).toFixed(1).toString() + "%)"}
            </div>
          </div>
        </div>
      </PanelSectionRow>
      <PanelSectionRow>
        <div className={FieldWithSeparator}>
          <div className={gamepadDialogClasses.FieldLabelRow}>
            <div className={gamepadDialogClasses.FieldLabel}>
            Max (Design)
            </div>
            <div className={gamepadDialogClasses.FieldChildren}>
            {(7.7 * chargeFullGlobal / 1000000).toFixed(1).toString() + " Wh (" + (100 * chargeFullGlobal / chargeDesignGlobal).toFixed(1).toString() + "%)"}
            </div>
          </div>
        </div>
      </PanelSectionRow>
      {/* Persistence */}
      <PanelSectionRow>
        <Toggle
          checked={persistGlobal}
          label="Persistent"
          description="Restores settings after an app or OS restart"
          onChange={(persist: boolean) => {
            console.log("Persist is now " + persist.toString());
            python.execute(python.setPersistent(persist));
            python.resolve(python.getPersistent(), setPersist);
          }}
        />
      </PanelSectionRow>
      <PanelSectionRow>
        <Toggle
          checked={perGameProfileGlobal}
          label="Use per-game profile"
          onChange={(p: boolean) => {
            console.log("Per game profile is now " + p.toString());
            python.execute(python.setPerGameProfile(p));
            python.resolve(python.getPerGameProfile(), setPerGameProfile);
            reload();
          }}
        />
      </PanelSectionRow>
      <PanelSectionRow>
        <div className={FieldWithSeparator}>
          <div className={gamepadDialogClasses.FieldLabelRow}>
            <div className={gamepadDialogClasses.FieldLabel}>
            Now Playing
            </div>
            <div className={gamepadDialogClasses.FieldChildren}>
            {gameGlobal}
            </div>
          </div>
        </div>
      </PanelSectionRow>
      {/* Version */}
      <div className={staticClasses.PanelSectionTitle}>
        Debug
      </div>
      <PanelSectionRow>
        <div className={FieldWithSeparator}>
          <div className={gamepadDialogClasses.FieldLabelRow}>
            <div className={gamepadDialogClasses.FieldLabel}>
            PowerTools
            </div>
            <div className={gamepadDialogClasses.FieldChildren}>
            v{versionGlobal}
            </div>
          </div>
        </div>
      </PanelSectionRow>
    </PanelSection>
  );
};

const DeckyPluginRouterTest: VFC = () => {
  return (
    <div style={{ marginTop: "50px", color: "white" }}>
      Hello World!
      <DialogButton onClick={() => {}}>
        Go to Store
      </DialogButton>
    </div>
  );
};

export default definePlugin((serverApi: ServerAPI) => {
  serverApi.routerHook.addRoute("/decky-plugin-test", DeckyPluginRouterTest, {
    exact: true,
  });

  return {
    title: <div className={staticClasses.Title}>PowerTools</div>,
    content: <Content serverAPI={serverApi} />,
    icon: <GiDrill />,
    onDismount() {
      console.log("PowerTools shutting down");
      clearInterval(periodicHook!);
      lifetimeHook.unregister();
      startHook.unregister();
      serverApi.routerHook.removeRoute("/decky-plugin-test");
    },
  };
});
