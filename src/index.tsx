import {
  ButtonItem,
  definePlugin,
  //DialogButton,
  //Menu,
  //MenuItem,
  PanelSection,
  PanelSectionRow,
  ServerAPI,
  //showContextMenu,
  staticClasses,
  //SliderField,
  ToggleField,
  //Dropdown,
  Field,
  //DropdownOption,
  //SingleDropdownOption,
  //NotchLabel
  //gamepadDialogClasses,
  //joinClassNames,
} from "decky-frontend-lib";
import { VFC, useState } from "react";
import { GiDrill } from "react-icons/gi";

//import * as python from "./python";
import * as backend from "./backend";
import { tr } from "usdpl-front";
import {
  BACKEND_INFO,
  DRIVER_INFO,

  LIMITS_INFO,

  CURRENT_BATT,
  CHARGE_RATE_BATT,
  CHARGE_MODE_BATT,
  CHARGE_NOW_BATT,
  CHARGE_FULL_BATT,
  CHARGE_DESIGN_BATT,

  ONLINE_CPUS,
  ONLINE_STATUS_CPUS,
  SMT_CPU,
  CLOCK_MIN_CPU,
  CLOCK_MAX_CPU,
  CLOCK_MIN_MAX_CPU,
  GOVERNOR_CPU,

  FAST_PPT_GPU,
  SLOW_PPT_GPU,
  CLOCK_MIN_GPU,
  CLOCK_MAX_GPU,
  SLOW_MEMORY_GPU,

  PERSISTENT_GEN,
  NAME_GEN,
} from "./consts";
import { set_value, get_value } from "usdpl-front";
import { Debug } from "./components/debug";
import { Gpu } from "./components/gpu";
import { Battery } from "./components/battery";
import { Cpus } from "./components/cpus";

var periodicHook: NodeJS.Timer | null = null;
var lifetimeHook: any = null;
var startHook: any = null;
var usdplReady = false;

type MinMax = {
  min: number | null;
  max: number | null;
}

function countCpus(statii: boolean[]): number {
  let count = 0;
  for (let i = 0; i < statii.length; i++) {
    if (statii[i]) {
      count += 1;
    }
  }
  return count;
}

function syncPlebClockToAdvanced() {
  const cpuCount = (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.count;
  const minClock = get_value(CLOCK_MIN_CPU);
  const maxClock = get_value(CLOCK_MAX_CPU);
  let clockArr = [];
  for (let i = 0; i < cpuCount; i++) {
    clockArr.push({
      min: minClock,
      max: maxClock,
    } as MinMax);
  }
  set_value(CLOCK_MIN_MAX_CPU, clockArr);
}

const reload = function() {
  if (!usdplReady) {return;}

  backend.resolve(backend.getLimits(), (limits) => {
    set_value(LIMITS_INFO, limits);
    console.debug("POWERTOOLS: got limits ", limits);
  });

  backend.resolve(backend.getBatteryCurrent(), (rate: number) => { set_value(CURRENT_BATT, rate) });
  backend.resolve_nullable(backend.getBatteryChargeRate(), (rate: number | null) => { set_value(CHARGE_RATE_BATT, rate) });
  backend.resolve_nullable(backend.getBatteryChargeMode(), (mode: string | null) => { set_value(CHARGE_MODE_BATT, mode) });
  backend.resolve(backend.getBatteryChargeNow(), (rate: number) => { set_value(CHARGE_NOW_BATT, rate) });
  backend.resolve(backend.getBatteryChargeFull(), (rate: number) => { set_value(CHARGE_FULL_BATT, rate) });
  backend.resolve(backend.getBatteryChargeDesign(), (rate: number) => { set_value(CHARGE_DESIGN_BATT, rate) });

  //backend.resolve(backend.getCpuCount(), (count: number) => { set_value(TOTAL_CPUS, count)});
  backend.resolve(backend.getCpusOnline(), (statii: boolean[]) => {
    set_value(ONLINE_STATUS_CPUS, statii);
    const count = countCpus(statii);
    set_value(ONLINE_CPUS, count);
    //set_value(SMT_CPU, statii.length > 3 && statii[0] == statii[1] && statii[2] == statii[3]);
  });
  backend.resolve(backend.getCpuSmt(), (smt: boolean) => {
    set_value(SMT_CPU, smt);
  });
  backend.resolve(backend.getCpuClockLimits(0), (limits: number[]) => {
    set_value(CLOCK_MIN_CPU, limits[0]);
    set_value(CLOCK_MAX_CPU, limits[1]);
    syncPlebClockToAdvanced();
  });
  backend.resolve(backend.getCpusGovernor(), (governors: string[]) => {
    set_value(GOVERNOR_CPU, governors);
    backend.log(backend.LogLevel.Info, "POWERTOOLS: Governors from backend " + governors.toString());
  });

  backend.resolve(backend.getGpuPpt(), (ppts: number[]) => {
    set_value(FAST_PPT_GPU, ppts[0]);
    set_value(SLOW_PPT_GPU, ppts[1]);
  });
  backend.resolve(backend.getGpuClockLimits(), (limits: number[]) => {
    set_value(CLOCK_MIN_GPU, limits[0]);
    set_value(CLOCK_MAX_GPU, limits[1]);
  });
  backend.resolve(backend.getGpuSlowMemory(), (status: boolean) => { set_value(SLOW_MEMORY_GPU, status) });

  backend.resolve(backend.getGeneralPersistent(), (value: boolean) => { set_value(PERSISTENT_GEN, value) });
  backend.resolve(backend.getGeneralSettingsName(), (name: string) => { set_value(NAME_GEN, name) });

  backend.resolve(backend.getInfo(), (info: string) => { set_value(BACKEND_INFO, info) });
  backend.resolve(backend.getDriverProviderName("gpu"), (driver: string) => { set_value(DRIVER_INFO, driver) });
};

// init USDPL WASM and connection to back-end
(async function(){
  await backend.initBackend();
  usdplReady = true;
  set_value(NAME_GEN, "Default");
  reload(); // technically this is only a load

  // register Steam callbacks
  //@ts-ignore
  lifetimeHook = SteamClient.GameSessions.RegisterForAppLifetimeNotifications((update) => {
      if (update.bRunning) {
          //backend.log(backend.LogLevel.Debug, "AppID " + update.unAppID.toString() + " is now running");
      } else {
          //backend.log(backend.LogLevel.Debug, "AppID " + update.unAppID.toString() + " is no longer running");
          backend.resolve(
            backend.loadGeneralDefaultSettings(),
            (ok: boolean) => {backend.log(backend.LogLevel.Debug, "Loading default settings ok? " + ok)}
          );
      }
  });
  //@ts-ignore
  startHook = SteamClient.Apps.RegisterForGameActionStart((actionType, id) => {
      //@ts-ignore
      let gameInfo: any = appStore.GetAppOverviewByGameID(id);
      // don't use gameInfo.appid, haha
      backend.resolve(
        backend.loadGeneralSettings(id.toString() + ".json", gameInfo.display_name),
        (ok: boolean) => {backend.log(backend.LogLevel.Debug, "Loading settings ok? " + ok)}
      );
  });

  backend.log(backend.LogLevel.Debug, "Registered PowerTools callbacks, hello!");
})();

const periodicals = function() {
  backend.resolve(backend.getBatteryCurrent(), (rate: number) => { set_value(CURRENT_BATT, rate) });
  backend.resolve(backend.getBatteryChargeNow(), (rate: number) => { set_value(CHARGE_NOW_BATT, rate) });
  backend.resolve(backend.getBatteryChargeFull(), (rate: number) => { set_value(CHARGE_FULL_BATT, rate) });

  backend.resolve(backend.getGeneralPersistent(), (value: boolean) => { set_value(PERSISTENT_GEN, value) });
  backend.resolve(backend.getGeneralSettingsName(), (name: string) => {
    const oldValue = get_value(NAME_GEN);
    set_value(NAME_GEN, name);
    if (name != oldValue) {
      reload();
    }
  });
};

const Content: VFC<{ serverAPI: ServerAPI }> = ({}) => {

  const [idc, reloadGUI] = useState<any>("/shrug");

  if (periodicHook != null) {
    clearInterval(periodicHook);
    periodicHook = null;
  }

  periodicHook = setInterval(function() {
      periodicals();
      reloadGUI("periodic" + (new Date()).getTime().toString());
  }, 1000);

  if (!usdplReady || !get_value(LIMITS_INFO)) {
    // Not translated on purpose (to avoid USDPL issues)
    return (
      <PanelSection>
        USDPL or PowerTools's backend did not start correctly!
        <ButtonItem
          layout="below"
          onClick={(_: MouseEvent) => {
            console.log("POWERTOOLS: manual reload after startup failure");
            reload();
          }}
        >
        Reload
        </ButtonItem>
      </PanelSection>
    )
  }

  return (
    <PanelSection>
      <Cpus idc={idc}/>

      <Gpu idc={idc}/>

      <Battery idc={idc}/>


      {/* Persistence */}
      <div className={staticClasses.PanelSectionTitle}>
        {tr("Miscellaneous")}
      </div>
      <PanelSectionRow>
        <ToggleField
          checked={get_value(PERSISTENT_GEN)}
          label={tr("Persistent Profile")}
          description={tr("Save profile and load it next time")}
          onChange={(persist: boolean) => {
            backend.log(backend.LogLevel.Debug, "Persist is now " + persist.toString());
            backend.resolve(
              backend.setGeneralPersistent(persist),
              (val: boolean) => {set_value(PERSISTENT_GEN, val)}
            );
          }}
        />
      </PanelSectionRow>
      <PanelSectionRow>
        <Field
          label={tr("Profile")}>
          {get_value(NAME_GEN)}
        </Field>
      </PanelSectionRow>

      <Debug idc={idc}/>

      <PanelSectionRow>
        <ButtonItem
          layout="below"
          onClick={(_: MouseEvent) => {
            backend.log(backend.LogLevel.Debug, "Loading default PowerTools settings");
            backend.resolve(
              backend.setGeneralPersistent(false),
              (val: boolean) => {
                set_value(PERSISTENT_GEN, val);
                backend.resolve(backend.loadGeneralSystemSettings(), (_) => {
                  reload();
                  backend.resolve(backend.waitForComplete(), (_) => {reloadGUI("LoadSystemDefaults")});
                });
              }
            );
          }}
        >
        {tr("Defaults")}
        </ButtonItem>
      </PanelSectionRow>
    </PanelSection>
  );
};

export default definePlugin((serverApi: ServerAPI) => {
  return {
    title: <div className={staticClasses.Title}>I'm a tool</div>,
    content: <Content serverAPI={serverApi} />,
    icon: <GiDrill />,
    onDismount() {
      backend.log(backend.LogLevel.Debug, "PowerTools shutting down");
      clearInterval(periodicHook!);
      periodicHook = null;
      lifetimeHook!.unregister();
      startHook!.unregister();
      serverApi.routerHook.removeRoute("/decky-plugin-test");
      backend.log(backend.LogLevel.Debug, "Unregistered PowerTools callbacks, so long and thanks for all the fish.");
    },
  };
});
