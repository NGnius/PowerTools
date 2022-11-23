import {
    ButtonItem,
    definePlugin,
    Dropdown,
    PanelSection,
    PanelSectionRow,
    ServerAPI,
    SingleDropdownOption,
    staticClasses,
    ToggleField,
    Router,
} from "decky-frontend-lib";
import { useEffect, useRef, useState, VFC } from "react";
import { GiDrill } from "react-icons/gi";
import { init_embedded, init_usdpl, target_usdpl, version_usdpl } from "usdpl-front";
import { BackendCalls, General, callBackend, getValue, setValue } from "./usdplFront";

import { FieldRow, SliderRow, ToggleRow } from "./Fields";
import { useBatteryReducer } from "./hooks/useBatteryReducer";
import { useCpuReducer } from "./hooks/useCpuReducer";
import { useGeneralReducer } from "./hooks/useGeneralReducer";
import { useGpuReducer } from "./hooks/useGpuReducer";
import { useInterval } from "./hooks/useInterval";
import { reload } from "./utilities/reload";
import { toPercentString } from "./utilities/toPercentString";

let lifetimeHook: Registerable | null = null;
let startHook: Registerable | null = null;
let usdplReady = false;

const USDPL_PORT = 44443;

// init USDPL WASM and connection to back-end
(async function () {
    await init_embedded();
    init_usdpl(USDPL_PORT);
    console.log("USDPL started for framework: " + target_usdpl());
    usdplReady = true;
    setValue(General.Name, "Default");

    // register Steam callbacks
    lifetimeHook = SteamClient.GameSessions.RegisterForAppLifetimeNotifications((update) => {
        if (update.bRunning) {
            //console.debug("AppID " + update.unAppID.toString() + " is now running");
        } else {
            //console.debug("AppID " + update.unAppID.toString() + " is no longer running");
            callBackend(BackendCalls.GeneralLoadDefaultSettings, []).then(([ok]) =>
                console.debug("Loading default settings ok? " + ok)
            );
        }
    });
    startHook = SteamClient.Apps.RegisterForGameActionStart((_, id) => {
        const gameInfo = appStore.GetAppOverviewByGameID(id);
        // don't use gameInfo.appid, haha
        callBackend(BackendCalls.GeneralLoadSettings, [id.toString() + ".json", gameInfo.display_name]).then(([ok]) =>
            console.debug("Loading settings ok? " + ok)
        );
    });

    console.debug("Registered PowerTools callbacks, hello!");
})();

const Content: VFC<{ serverAPI: ServerAPI }> = () => {
    const [batteryState, batteryDispatch, batteryRefetch] = useBatteryReducer();
    const [generalState, generalDispatch, generalRefetch] = useGeneralReducer();
    const [cpuState, cpuDispatch, cpuRefetch] = useCpuReducer();
    const [gpuState, gpuDispatch, gpuRefetch] = useGpuReducer();
    const reloadInflightRef = useRef(false); // don't re-render when inflight flag flips
    const [eggCount, setEggCount] = useState(0);

    // load data on initial render
    useEffect(() => {
        reloadInflightRef.current = true;
        reload({ usdplReady, fullReload: true, smtAllowed: cpuState.smtAllowed }).then(() => {
            batteryRefetch();
            generalRefetch();
            cpuRefetch();
            gpuRefetch();
            reloadInflightRef.current = false;
        });
    }, []);

    // poll BE for updates
    useInterval(() => {
        if (reloadInflightRef.current) {
            return; // exit early if reloading
        }
        reloadInflightRef.current = true;
        reload({ usdplReady, fullReload: false, smtAllowed: cpuState.smtAllowed }).then(() => {
            batteryRefetch();
            generalRefetch();
            cpuRefetch();
            gpuRefetch();
            reloadInflightRef.current = false;
        });
    }, 5000);

    const limits = getValue(General.LimitsAll);

    const governorOptions: SingleDropdownOption[] = (
        cpuState.advancedCpuIndex ? limits.cpu.cpus[cpuState.advancedCpuIndex].governors : []
    ).map((elem) => {
        return {
            data: elem,
            label: <span>{elem}</span>,
        };
    });

    const chargeModeOptions: SingleDropdownOption[] = limits.battery.charge_modes.map((elem) => {
        return {
            data: elem,
            label: <span>{elem}</span>,
        };
    });

    const isNerd = eggCount % 10 === 9;

    const cpuMinmaxMin = cpuState.CPUs_minmax_clocks[cpuState.advancedCpuIndex].min;
    const cpuMinmaxMax = cpuState.CPUs_minmax_clocks[cpuState.advancedCpuIndex].max;
    const cpu0Limits = limits.cpu.cpus[0];

    const chargeNow =
        batteryState.BATTERY_charge_now !== null &&
        batteryState.BATTERY_charge_full !== null &&
        toPercentString(batteryState.BATTERY_charge_now, batteryState.BATTERY_charge_full, "Wh");
    const chargeMax =
        batteryState.BATTERY_charge_full !== null &&
        batteryState.BATTERY_charge_design !== null &&
        toPercentString(batteryState.BATTERY_charge_full, batteryState.BATTERY_charge_design, "Wh");

    return (
        <PanelSection>
            {/* CPU */}
            <div className={staticClasses.PanelSectionTitle}>CPU</div>
            <PanelSectionRow>
                <ToggleField
                    checked={!!cpuState.advancedMode}
                    label="Advanced"
                    description="Enables per-thread configuration"
                    onChange={(toggle) => cpuDispatch(["advancedModeToggle", toggle])}
                />
            </PanelSectionRow>
            {!cpuState.advancedMode ? (
                <>
                    {/* CPU plebeian mode */}
                    {cpuState.smtAllowed && (
                        <ToggleRow
                            checked={!!cpuState.CPUs_SMT}
                            label="setSmt"
                            description="Enables odd-numbered CPUs"
                            onChange={(smt) => cpuDispatch(["setSmt", smt])}
                        />
                    )}
                    {typeof cpuState.total_cpus === "number" && (
                        <SliderRow
                            label="Threads"
                            value={cpuState.CPUs_online ?? -1}
                            step={1}
                            max={
                                cpuState.CPUs_SMT || !cpuState.smtAllowed
                                    ? cpuState.total_cpus
                                    : cpuState.total_cpus / 2
                            }
                            min={1}
                            showValue={true}
                            onChange={(cpus) => cpuDispatch(["immediate", cpus])}
                        />
                    )}
                    <ToggleRow
                        checked={cpuState.CPUs_min_clock !== null && cpuState.CPUs_max_clock !== null}
                        label="Frequency Limits"
                        description="Set bounds on clock speed"
                        onChange={(toggle) => cpuDispatch(["freqToggle", toggle])}
                    />
                    {cpuState.CPUs_min_clock !== null && cpu0Limits.clock_min_limits && (
                        <SliderRow
                            label="Minimum (MHz)"
                            value={cpuState.CPUs_min_clock}
                            max={cpu0Limits.clock_min_limits.max}
                            min={cpu0Limits.clock_min_limits.min}
                            step={cpu0Limits.clock_step}
                            showValue={true}
                            disabled={cpuState.CPUs_min_clock === null}
                            onChange={(freq) => cpuDispatch(["minFreq", freq])}
                        />
                    )}
                    {cpuState.CPUs_max_clock !== null && cpu0Limits.clock_max_limits && (
                        <SliderRow
                            label="Maximum (MHz)"
                            value={cpuState.CPUs_max_clock}
                            max={cpu0Limits.clock_max_limits.max}
                            min={cpu0Limits.clock_max_limits.min}
                            step={cpu0Limits.clock_step}
                            showValue={true}
                            disabled={cpuState.CPUs_max_clock === null}
                            onChange={(freq) => cpuDispatch(["maxFreq", freq])}
                        />
                    )}
                </>
            ) : (
                <>
                    {/* CPU advanced mode */}
                    <SliderRow
                        label="Selected CPU"
                        value={cpuState.advancedCpuIndex + 1}
                        step={1}
                        max={cpuState.total_cpus}
                        min={1}
                        showValue={true}
                        onChange={(selectedCpuDisplayValue) => {
                            const selectedCpu = selectedCpuDisplayValue - 1;
                            cpuDispatch(["advancedModeCpuSelector", selectedCpu]);
                        }}
                    />
                    <ToggleRow
                        checked={cpuState.CPUs_status_online[cpuState.advancedCpuIndex]}
                        label="Online"
                        description="Allow the CPU thread to do work"
                        onChange={(status) => cpuDispatch(["setSmtAdvanced", status])}
                    />
                    <ToggleRow
                        checked={cpuMinmaxMin !== null || cpuMinmaxMax !== null}
                        label="Frequency Limits"
                        description="Set bounds on clock speed"
                        onChange={(value) => cpuDispatch(["freqToggleAdvanced", value])}
                    />
                    {cpu0Limits.clock_min_limits !== null && cpuMinmaxMin !== null && (
                        <SliderRow
                            label="Minimum (MHz)"
                            value={cpuMinmaxMin}
                            max={cpu0Limits.clock_min_limits.max}
                            min={cpu0Limits.clock_min_limits.min}
                            step={cpu0Limits.clock_step}
                            showValue={true}
                            disabled={cpuMinmaxMin === null}
                            onChange={(freq) => cpuDispatch(["minFreqAdvanced", freq])}
                        />
                    )}
                    {cpu0Limits.clock_max_limits !== null && cpuMinmaxMax !== null && (
                        <SliderRow
                            label="Maximum (MHz)"
                            value={cpuMinmaxMax}
                            max={cpu0Limits.clock_max_limits.max}
                            min={cpu0Limits.clock_max_limits.min}
                            step={cpu0Limits.clock_step}
                            showValue={true}
                            disabled={cpuMinmaxMax === null}
                            onChange={(freq) => cpuDispatch(["maxFreqAdvanced", freq])}
                        />
                    )}
                    {cpuState.advancedCpuIndex !== null && governorOptions.length !== null && (
                        <FieldRow label="Governor">
                            <Dropdown
                                menuLabel="Governor"
                                rgOptions={governorOptions}
                                selectedOption={governorOptions.find(
                                    (val) => val.data === cpuState.CPUs_governor[cpuState.advancedCpuIndex]
                                )}
                                strDefaultLabel={cpuState.CPUs_governor[cpuState.advancedCpuIndex]}
                                onChange={({ data }: SingleDropdownOption) => cpuDispatch(["governor", data])}
                            />
                        </FieldRow>
                    )}
                </>
            )}

            {/* GPU */}
            <div className={staticClasses.PanelSectionTitle}>GPU</div>
            {(limits.gpu.fast_ppt_limits !== null || limits.gpu.slow_ppt_limits !== null) && (
                <ToggleRow
                    checked={gpuState.GPU_slowPPT !== null || gpuState.GPU_fastPPT !== null}
                    label="PowerPlay Limits"
                    description="Override APU TDP settings"
                    onChange={(toggle: boolean) => gpuDispatch(["pptToggle", toggle])}
                />
            )}
            {gpuState.GPU_slowPPT !== null && limits.gpu.slow_ppt_limits && (
                <SliderRow
                    label="SlowPPT (W)"
                    value={gpuState.GPU_slowPPT}
                    max={limits.gpu.slow_ppt_limits.max}
                    min={limits.gpu.slow_ppt_limits.min}
                    step={limits.gpu.ppt_step}
                    showValue={true}
                    disabled={gpuState.GPU_slowPPT === null}
                    onChange={(ppt) => gpuDispatch(["slowPPT", ppt])}
                />
            )}
            {gpuState.GPU_fastPPT !== null && limits.gpu.fast_ppt_limits && (
                <SliderRow
                    label="FastPPT (W)"
                    value={gpuState.GPU_fastPPT}
                    max={limits.gpu.fast_ppt_limits.max}
                    min={limits.gpu.fast_ppt_limits.min}
                    step={limits.gpu.ppt_step}
                    showValue={true}
                    disabled={gpuState.GPU_fastPPT === null}
                    onChange={(ppt) => gpuDispatch(["fastPPT", ppt])}
                />
            )}
            {(limits.gpu.clock_min_limits || limits.gpu.clock_max_limits) && (
                <ToggleRow
                    checked={gpuState.GPU_min_clock !== null || gpuState.GPU_max_clock !== null}
                    label="Frequency Limits"
                    description="Override bounds on gpu clock"
                    onChange={(toggle) => gpuDispatch(["freqToggle", toggle])}
                />
            )}
            {gpuState.GPU_min_clock !== null && limits.gpu.clock_min_limits && (
                <SliderRow
                    label="Minimum (MHz)"
                    value={gpuState.GPU_min_clock}
                    max={limits.gpu.clock_min_limits.max}
                    min={limits.gpu.clock_min_limits.min}
                    step={limits.gpu.clock_step}
                    showValue={true}
                    disabled={gpuState.GPU_min_clock === null}
                    onChange={(val) => gpuDispatch(["minClock", val])}
                />
            )}
            {gpuState.GPU_max_clock !== null && limits.gpu.clock_max_limits && (
                <SliderRow
                    label="Maximum (MHz)"
                    value={gpuState.GPU_max_clock}
                    max={limits.gpu.clock_max_limits.max}
                    min={limits.gpu.clock_max_limits.min}
                    step={limits.gpu.clock_step}
                    showValue={true}
                    disabled={gpuState.GPU_max_clock === null}
                    onChange={(val) => gpuDispatch(["maxClock", val])}
                />
            )}
            {limits.gpu.memory_control_capable && (
                <ToggleRow
                    checked={gpuState.GPU_slow_memory ?? false}
                    label="Downclock Memory"
                    description="Force RAM into low-power mode"
                    onChange={(value) => gpuDispatch(["slowMemory", value])}
                />
            )}
            {/* Battery */}
            <div className={staticClasses.PanelSectionTitle}>Battery</div>
            {chargeNow && (
                <FieldRow label="Now (Charge)" onClick={() => setEggCount((prev) => prev + 1)} focusable={false}>
                    {chargeNow}
                </FieldRow>
            )}
            {chargeMax && (
                <FieldRow label="Max (Design)" onClick={() => setEggCount((prev) => prev + 1)} focusable={false}>
                    {chargeMax}
                </FieldRow>
            )}
            {limits.battery.charge_current && (
                <>
                    <ToggleRow
                        checked={batteryState.BATTERY_charge_rate !== null}
                        label="Charge Current Limits"
                        description="Control battery charge rate when awake"
                        onChange={(toggle) => batteryDispatch(["chargeRateToggle", toggle])}
                    />
                    <SliderRow
                        label="Maximum (mA)"
                        value={batteryState.BATTERY_charge_rate ?? -1}
                        max={limits.battery.charge_current.max}
                        min={limits.battery.charge_current.min}
                        step={limits.battery.charge_current_step}
                        showValue={true}
                        disabled={batteryState.BATTERY_charge_rate === null}
                        onChange={(val) => batteryDispatch(["chargeRate", val])}
                    />
                </>
            )}
            {chargeModeOptions.length !== 0 && (
                <>
                    <ToggleRow
                        checked={batteryState.BATTERY_charge_mode !== null}
                        label="Charge Mode"
                        description="Force battery charge mode"
                        onChange={(toggle) =>
                            batteryDispatch(["chargeModeToggle", { toggle, value: chargeModeOptions[0].data }])
                        }
                    />
                    {batteryState.BATTERY_charge_mode !== null && (
                        <FieldRow label="Mode">
                            <Dropdown
                                menuLabel="Charge Mode"
                                rgOptions={chargeModeOptions}
                                selectedOption={chargeModeOptions.find(
                                    (val) => val.data === batteryState.BATTERY_charge_mode
                                )}
                                strDefaultLabel={batteryState.BATTERY_charge_mode}
                                onChange={(elem) => batteryDispatch(["chargeMode", elem.data])}
                            />
                        </FieldRow>
                    )}
                </>
            )}
            <FieldRow label="Current" onClick={() => setEggCount((prev) => prev + 1)} focusable={false}>
                {batteryState.BATTERY_current_now} mA
            </FieldRow>
            {/* Persistence */}
            <div className={staticClasses.PanelSectionTitle}>Miscellaneous</div>
            <ToggleRow
                checked={generalState.GENERAL_persistent ?? false}
                label="Persistent"
                description="Save profile and load it next time"
                onChange={(persist) => generalDispatch(["setPersistent", persist])}
            />
            <FieldRow label="Profile">{generalState.GENERAL_name ?? "NULL"}</FieldRow>
            {/* Version Info */}
            <div className={staticClasses.PanelSectionTitle}>
                Debug
                {isNerd ? "Ha! Nerd" : "Debug"}
            </div>
            <FieldRow
                label={isNerd ? "PowerTools" : "Native"}
                onClick={() => {
                    // you know you're bored and/or conceited when you spend time adding an easter egg
                    // that just sends people to your own project's repo
                    setEggCount((prev) => prev + 1);
                    if (isNerd) Router.NavigateToExternalWeb("https://github.com/NGnius/PowerTools");
                }}
            >
                {isNerd ? "by NGnius" : generalState.V_INFO}
            </FieldRow>
            <FieldRow label="Framework">{target_usdpl()}</FieldRow>
            <FieldRow
                label="USDPL"
                onClick={() => {
                    // you know you're bored and/or conceited when you spend time adding an easter egg
                    // that just sends people to your own project's repo
                    setEggCount((prev) => prev + 1);
                    if (isNerd) Router.NavigateToExternalWeb("https://github.com/NGnius/usdpl-rs");
                }}
            >
                v{version_usdpl()}
            </FieldRow>
            <FieldRow label="USDPL">{`v${version_usdpl()}`}</FieldRow>
            {eggCount % 10 === 9 && (
                <PanelSectionRow>
                    <ButtonItem layout="below" onClick={() => generalDispatch(["idk"])}>
                        ???
                    </ButtonItem>
                </PanelSectionRow>
            )}
            <ButtonItem
                layout="below"
                onClick={() =>
                    generalDispatch([
                        "loadSystemDefaults",
                        () => reload({ usdplReady: true, fullReload: true, smtAllowed: cpuState.smtAllowed }),
                    ])
                }
            >
                Defaults
            </ButtonItem>
        </PanelSection>
    );
};

export default definePlugin((serverApi: ServerAPI) => {
    return {
        title: <div className={staticClasses.Title}>PowerTools</div>,
        content: <Content serverAPI={serverApi} />,
        icon: <GiDrill />,
        onDismount() {
            console.debug("PowerTools shutting down");
            lifetimeHook?.unregister?.();
            startHook?.unregister?.();
            serverApi.routerHook.removeRoute("/decky-plugin-test");
            console.debug("Unregistered PowerTools callbacks, goodbye.");
        },
    };
});
