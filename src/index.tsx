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
import { useCallback, useEffect, useRef, useState, VFC } from "react";
import { GiDrill } from "react-icons/gi";

import { init_embedded, init_usdpl, target_usdpl, version_usdpl } from "usdpl-front";
import { call_backend, get_value, set_value } from "./utilities/augmentedUsdplFront";

import { FieldRow, SliderRow, ToggleRow } from "./Fields";
import { useBatteryReducer } from "./hooks/useBatteryReducer";
import { useCpuReducer } from "./hooks/useCpuReducer";
import { useGeneralReducer } from "./hooks/useGeneralReducer";
import { useGpuReducer } from "./hooks/useGpuReducer";
import { useInterval } from "./hooks/useInterval";
import { periodicals } from "./utilities/periodicals";
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
    set_value("GENERAL_name", "Default");
    // reload(); // technically this is only a load
    // reload moved to body of Content component so that reload is called within
    // the react component lifecycle

    // register Steam callbacks
    lifetimeHook = SteamClient.GameSessions.RegisterForAppLifetimeNotifications((update) => {
        if (update.bRunning) {
            //console.debug("AppID " + update.unAppID.toString() + " is now running");
        } else {
            //console.debug("AppID " + update.unAppID.toString() + " is no longer running");
            call_backend("GENERAL_load_default_settings", []).then(([ok]) =>
                console.debug("Loading default settings ok? " + ok)
            );
        }
    });
    startHook = SteamClient.Apps.RegisterForGameActionStart((_, id) => {
        const gameInfo = appStore.GetAppOverviewByGameID(id);
        // don't use gameInfo.appid, haha
        call_backend("GENERAL_load_settings", [id.toString() + ".json", gameInfo.display_name]).then(([ok]) =>
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
    const [, isLoading] = useState(true); // maybe initialize app with a loading state?
    const [eggCount, setEggCount] = useState(0);
    const smtAllowedRef = useRef(!!cpuState.smtAllowed);
    smtAllowedRef.current = !!cpuState.smtAllowed;
    const [mounted, setMounted] = useState(false);

    const initialFetch = useCallback(async () => {
        await reload(usdplReady, smtAllowedRef);
        batteryRefetch();
        generalRefetch();
        cpuRefetch();
        gpuRefetch();
        isLoading(false);
    }, []);

    if (!mounted) {
        initialFetch();
    }

    // fetch data on initial render
    useEffect(() => {
        setMounted(true);
    }, []);

    // poll BE for updates
    useInterval(() => {
        periodicals(usdplReady, smtAllowedRef);
        batteryRefetch();
        generalRefetch();
        cpuRefetch();
        gpuRefetch();
    }, 5000);

    const limits = get_value("LIMITS_all");

    const governorOptions: SingleDropdownOption[] = (
        cpuState.advancedModeCpu ? cpuState.LIMITS_all.cpu.cpus[cpuState.advancedModeCpu].governors : []
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

    const advancedModeCpu = cpuState.advancedModeCpu ?? 0;
    const cpuMinmaxMin = cpuState.CPUs_minmax_clocks[advancedModeCpu].min;
    const cpuMinmaxMax = cpuState.CPUs_minmax_clocks[advancedModeCpu].max;
    const cpuLimits = limits.cpu.cpus[0];
    const isNerd = eggCount % 10 === 9;

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
                            label="SMT"
                            description="Enables odd-numbered CPUs"
                            onChange={(smt) => cpuDispatch(["SMT", smt])}
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
                            onChange={(cpus) => cpuDispatch(["CPUsImmediate", cpus])}
                        />
                    )}
                    <ToggleRow
                        checked={cpuState.CPUs_min_clock !== null && cpuState.CPUs_max_clock !== null}
                        label="Frequency Limits"
                        description="Set bounds on clock speed"
                        onChange={(toggle) => cpuDispatch(["CPUFreqToggle", toggle])}
                    />
                    {cpuState.CPUs_min_clock !== null && cpuLimits.clock_min_limits && (
                        <SliderRow
                            label="Minimum (MHz)"
                            value={cpuState.CPUs_min_clock}
                            max={cpuLimits.clock_min_limits.max}
                            min={cpuLimits.clock_min_limits.min}
                            step={cpuLimits.clock_step}
                            showValue={true}
                            disabled={cpuState.CPUs_min_clock === null}
                            onChange={(freq) => cpuDispatch(["CPUMinFreq", freq])}
                        />
                    )}
                    {cpuState.CPUs_max_clock !== null && cpuLimits.clock_max_limits && (
                        <SliderRow
                            label="Maximum (MHz)"
                            value={cpuState.CPUs_max_clock}
                            max={cpuLimits.clock_max_limits.max}
                            min={cpuLimits.clock_max_limits.min}
                            step={cpuLimits.clock_step}
                            showValue={true}
                            disabled={cpuState.CPUs_max_clock === null}
                            onChange={(freq) => cpuDispatch(["CPUMaxFreq", freq])}
                        />
                    )}
                </>
            ) : (
                <>
                    {/* CPU advanced mode */}
                    <SliderRow
                        label="Selected CPU"
                        // this should probably not be translated and instead be kept zero-based to match common
                        // patterns for cpu/thread count indexing?
                        value={advancedModeCpu + 1}
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
                        checked={cpuState.CPUs_status_online[advancedModeCpu]}
                        // checked={cpuState.CPUs_online[advancedModeCpu]}
                        label="Online"
                        description="Allow the CPU thread to do work"
                        onChange={(status) => cpuDispatch(["SMTAdvanced", status])}
                    />
                    <ToggleRow
                        checked={cpuMinmaxMin !== null || cpuMinmaxMax !== null}
                        label="Frequency Limits"
                        description="Set bounds on clock speed"
                        onChange={(value) => cpuDispatch(["CPUFreqToggleAdvanced", value])}
                    />
                    {cpuLimits.clock_min_limits !== null && cpuMinmaxMin !== null && (
                        <SliderRow
                            label="Minimum (MHz)"
                            value={cpuMinmaxMin}
                            max={cpuLimits.clock_min_limits.max}
                            min={cpuLimits.clock_min_limits.min}
                            step={cpuLimits.clock_step}
                            showValue={true}
                            disabled={cpuMinmaxMin === null}
                            onChange={(freq) => cpuDispatch(["CPUMinFreqAdvanced", freq])}
                        />
                    )}
                    {cpuLimits.clock_max_limits !== null && cpuMinmaxMax !== null && (
                        <SliderRow
                            label="Maximum (MHz)"
                            value={cpuMinmaxMax}
                            max={cpuLimits.clock_max_limits.max}
                            min={cpuLimits.clock_max_limits.min}
                            step={cpuLimits.clock_step}
                            showValue={true}
                            disabled={cpuMinmaxMax === null}
                            onChange={(freq) => cpuDispatch(["CPUMaxFreqAdvanced", freq])}
                        />
                    )}
                    {advancedModeCpu !== null && governorOptions.length !== null && (
                        <FieldRow label="Governor">
                            <Dropdown
                                menuLabel="Governor"
                                rgOptions={governorOptions}
                                selectedOption={governorOptions.find(
                                    (val) => val.data === cpuState.CPUs_governor[advancedModeCpu]
                                )}
                                strDefaultLabel={cpuState.CPUs_governor[advancedModeCpu]}
                                onChange={({ data }: SingleDropdownOption) => cpuDispatch(["CPUGovernor", data])}
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
                    onChange={(toggle: boolean) => gpuDispatch(["GPUPPTToggle", toggle ? 15000000 : null])}
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
                    onChange={(ppt) => gpuDispatch(["GPUSlowPPT", ppt])}
                />
            )}
            {gpuState.GPU_fastPPT !== null && limits.gpu.fast_ppt_limits?.max && limits.gpu.fast_ppt_limits.min && (
                <SliderRow
                    label="FastPPT (W)"
                    value={gpuState.GPU_fastPPT}
                    max={limits.gpu.fast_ppt_limits.max}
                    min={limits.gpu.fast_ppt_limits.min}
                    step={limits.gpu.ppt_step}
                    showValue={true}
                    disabled={gpuState.GPU_fastPPT === null}
                    onChange={(ppt) => gpuDispatch(["GPUFastPPT", ppt])}
                />
            )}
            {(limits.gpu.clock_min_limits !== null || limits.gpu.clock_max_limits !== null) && (
                <ToggleRow
                    checked={gpuState.GPU_min_clock !== null || gpuState.GPU_max_clock !== null}
                    label="Frequency Limits"
                    description="Override bounds on gpu clock"
                    onChange={(toggle) => gpuDispatch(["GPUFreqToggle", toggle])}
                />
            )}
            {gpuState.GPU_min_clock !== null && limits.gpu.clock_min_limits !== null && (
                <SliderRow
                    label="Minimum (MHz)"
                    value={gpuState.GPU_min_clock}
                    max={limits.gpu.clock_min_limits.max}
                    min={limits.gpu.clock_min_limits.min}
                    step={limits.gpu.clock_step}
                    showValue={true}
                    disabled={gpuState.GPU_min_clock === null}
                    onChange={(val) => gpuDispatch(["GPUMinClock", val])}
                />
            )}
            {gpuState.GPU_max_clock !== null && limits.gpu.clock_max_limits !== null && (
                <SliderRow
                    label="Maximum (MHz)"
                    value={gpuState.GPU_max_clock}
                    max={limits.gpu.clock_max_limits.max}
                    min={limits.gpu.clock_max_limits.min}
                    step={limits.gpu.clock_step}
                    showValue={true}
                    disabled={gpuState.GPU_max_clock === null}
                    onChange={(val) => gpuDispatch(["GPUMaxClock", val])}
                />
            )}
            {limits.gpu.memory_control_capable && (
                <ToggleRow
                    checked={gpuState.GPU_slow_memory ?? false}
                    label="Downclock Memory"
                    description="Force RAM into low-power mode"
                    onChange={(value) => gpuDispatch(["GPUSlowMemory", value])}
                />
            )}

            {/* Battery */}
            <div className={staticClasses.PanelSectionTitle}>Battery</div>
            {batteryState.BATTERY_charge_now !== null && batteryState.BATTERY_charge_full !== null && (
                <FieldRow label="Now (Charge)" onClick={() => setEggCount((prev) => prev + 1)} focusable={false}>
                    {toPercentString(batteryState.BATTERY_charge_now, batteryState.BATTERY_charge_full, "Wh")}
                </FieldRow>
            )}
            {batteryState.BATTERY_charge_full !== null && batteryState.BATTERY_charge_design !== null && (
                <FieldRow label="Max (Design)" onClick={() => setEggCount((prev) => prev + 1)} focusable={false}>
                    {toPercentString(batteryState.BATTERY_charge_full, batteryState.BATTERY_charge_design, "Wh")}
                </FieldRow>
            )}
            {limits.battery.charge_current !== null && (
                <>
                    <ToggleRow
                        checked={batteryState.BATTERY_charge_rate !== null}
                        label="Charge Current Limits"
                        description="Control battery charge rate when awake"
                        onChange={(toggle) => batteryDispatch(["BATTChargeRateToggle", toggle])}
                    />
                    <SliderRow
                        label="Maximum (mA)"
                        value={batteryState.BATTERY_charge_rate ?? -1}
                        max={limits.battery.charge_current.max}
                        min={limits.battery.charge_current.min}
                        step={limits.battery.charge_current_step}
                        showValue={true}
                        disabled={batteryState.BATTERY_charge_rate === null}
                        onChange={(val) => batteryDispatch(["BATTChargeRate", val])}
                    />
                </>
            )}

            {chargeModeOptions.length !== 0 && (
                <PanelSectionRow>
                    <ToggleField
                        checked={batteryState.BATTERY_charge_mode !== null}
                        label="Charge Mode"
                        description="Force battery charge mode"
                        onChange={(toggle) =>
                            batteryDispatch(["BATTChargeModeToggle", { toggle, value: chargeModeOptions[0].data }])
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
                                onChange={(elem) => batteryDispatch(["BATTChargeMode", elem.data])}
                            />
                        </FieldRow>
                    )}
                </PanelSectionRow>
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
                onChange={(persist) => generalDispatch(["SetPersistent", persist])}
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
            {eggCount % 10 == 9 && (
                <PanelSectionRow>
                    <ButtonItem layout="below" onClick={(_: MouseEvent) => backend.idk}>
                        ???
                    </ButtonItem>
                </PanelSectionRow>
            )}

            <ButtonItem layout="below" onClick={() => generalDispatch(["LoadSystemDefaults"])}>
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
