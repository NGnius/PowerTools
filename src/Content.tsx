import {
    Dropdown,
    PanelSection,
    PanelSectionRow,
    Router,
    ServerAPI,
    SingleDropdownOption,
    ToggleField,
} from "decky-frontend-lib";
import { useCallback, useEffect, useRef, useState, VFC } from "react";

import type { GlobalRefs } from "./index";
import { ButtonRow, FieldRow, SliderRow, ToggleRow /* , CpuGrid */ } from "./Fields";
import { useBatteryReducer } from "./hooks/useBatteryReducer";
import { useCpuReducer } from "./hooks/useCpuReducer";
import { useGeneralReducer } from "./hooks/useGeneralReducer";
import { useGpuReducer } from "./hooks/useGpuReducer";
import { intervalHookFactory } from "./hooks/useInterval";
import { isNull, notNull, toPercentString } from "./utilities/helpers";
import { reload } from "./utilities/reload";
import { SETTINGS_LIMITS } from "./utilities/settingsLimits";
import { BACKEND_CALLS, callBackend, targetUsdpl, versionUsdpl } from "./usdplFront";

const useInterval = intervalHookFactory(5000);

export const Content: VFC<{
    serverAPI: ServerAPI;
    globalRefs: GlobalRefs;
}> = ({ globalRefs }) => {
    const [batteryState, batteryDispatch] = useBatteryReducer();
    const [generalState, generalDispatch] = useGeneralReducer();
    const [cpuState, cpuDispatch] = useCpuReducer();
    const [gpuState, gpuDispatch] = useGpuReducer();
    const [inflight, setInflight] = useState(false);
    const inflightRef = useRef(inflight); // don't re-render when inflight flag flips
    inflightRef.current = inflight;
    const [eggCount, setEggCount] = useState(0);
    const { usdplReady } = globalRefs;
    const smtAllowed = cpuState.smtAllowed || false;

    const reloadCb = useCallback(() => {
        if (usdplReady) {
            setInflight(true);
            reload({ usdplReady, fullReload: true, smtAllowed }).then(() => {
                setInflight(false);
            });
        }
    }, [smtAllowed, usdplReady]);

    // load data on initial render
    useEffect(reloadCb, [reloadCb]);

    // poll BE for updates
    useInterval(() => (inflightRef.current ? undefined : reloadCb()), [reloadCb]);

    const governorOptions = (
        cpuState.advancedCpuIndex ? SETTINGS_LIMITS.cpu.cpus[cpuState.advancedCpuIndex].governors : []
    ).map((elem) => ({
        data: elem,
        label: <span>{elem}</span>,
    }));

    const chargeModeOptions = SETTINGS_LIMITS.battery.charge_modes.map((elem) => ({
        data: elem,
        label: <span>{elem}</span>,
    }));

    const isNerd = eggCount % 10 === 9;

    const cpuMinmaxMin = cpuState.CPUs_minmax_clocks[cpuState.advancedCpuIndex].min;
    const cpuMinmaxMax = cpuState.CPUs_minmax_clocks[cpuState.advancedCpuIndex].max;
    const cpu0Limits = SETTINGS_LIMITS.cpu.cpus[0];

    const chargeNow =
        (notNull(batteryState.BATTERY_charge_now) &&
            notNull(batteryState.BATTERY_charge_full) &&
            toPercentString(batteryState.BATTERY_charge_now, batteryState.BATTERY_charge_full, "Wh")) ||
        "CHARGE_NOW";
    const chargeMax =
        (notNull(batteryState.BATTERY_charge_full) &&
            notNull(batteryState.BATTERY_charge_design) &&
            toPercentString(batteryState.BATTERY_charge_full, batteryState.BATTERY_charge_design, "Wh")) ||
        "CHARGE_MAX";

    const [intv, setIntv] = useState({});

    useInterval(() => {
        async function cb() {
            const [BatteryCurrentNow] = await callBackend(BACKEND_CALLS.BatteryCurrentNow, []);
            const [BatteryChargeNow] = await callBackend(BACKEND_CALLS.BatteryChargeNow, []);
            const [BatteryChargeFull] = await callBackend(BACKEND_CALLS.BatteryChargeFull, []);
            const [GeneralGetPersistent] = await callBackend(BACKEND_CALLS.GeneralGetPersistent, []);
            setIntv({
                BatteryCurrentNow,
                BatteryChargeNow,
                BatteryChargeFull,
                GeneralGetPersistent,
            });
        }
        cb();
    }, []);

    return (
        <>
            <PanelSection title="Backend Values">
                <dl>
                    <dt>INTV</dt>
                    <dd>{JSON.stringify(intv)}</dd>
                    <dt>LIMITS_all</dt>
                    <dd>{JSON.stringify(SETTINGS_LIMITS)}</dd>
                    <dt>BATTERY</dt>
                    <dd>{JSON.stringify(batteryState)}</dd>
                </dl>
            </PanelSection>
            <PanelSection title="CPU">
                <PanelSectionRow>
                    <ToggleField
                        bottomSeparator="none"
                        checked={!!cpuState.advancedMode}
                        label="Advanced"
                        description="Enables per-thread configuration"
                        onChange={(toggle) => cpuDispatch(["advancedModeToggle", toggle])}
                    />
                </PanelSectionRow>
                {/* <CpuGrid cores={cpuState.CPUs_status_online} /> */}
                {!cpuState.advancedMode ? (
                    <>
                        {/* CPU plebeian mode */}
                        {smtAllowed && (
                            <ToggleRow
                                checked={!!cpuState.CPUs_SMT}
                                label="setSmt"
                                description="Enables odd-numbered CPUs"
                                onChange={(smt) => cpuDispatch(["setSmt", smt])}
                            />
                        )}
                        <SliderRow
                            label="Threads"
                            value={cpuState.CPUs_online}
                            step={1}
                            max={cpuState.CPUs_SMT || !smtAllowed ? cpuState.total_cpus : cpuState.total_cpus / 2}
                            min={1}
                            showValue={true}
                            onChange={(cpus) => cpuDispatch(["immediate", cpus])}
                        />
                        <ToggleRow
                            checked={notNull(cpuState.CPUs_min_clock) && notNull(cpuState.CPUs_max_clock)}
                            label="Frequency Limits"
                            description="Set bounds on clock speed"
                            onChange={(toggle) => cpuDispatch(["freqToggle", toggle])}
                        />
                        {notNull(cpuState.CPUs_min_clock) && (
                            <SliderRow
                                label="Minimum (MHz)"
                                value={cpuState.CPUs_min_clock}
                                max={cpu0Limits.clock_min_limits.max}
                                min={cpu0Limits.clock_min_limits.min}
                                step={cpu0Limits.clock_step}
                                showValue={true}
                                disabled={isNull(cpuState.CPUs_min_clock)}
                                onChange={(freq) => cpuDispatch(["minFreq", freq])}
                            />
                        )}
                        {notNull(cpuState.CPUs_max_clock) && (
                            <SliderRow
                                label="Maximum (MHz)"
                                value={cpuState.CPUs_max_clock}
                                max={cpu0Limits.clock_max_limits.max}
                                min={cpu0Limits.clock_max_limits.min}
                                step={cpu0Limits.clock_step}
                                showValue={true}
                                disabled={isNull(cpuState.CPUs_max_clock)}
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
                            checked={notNull(cpuMinmaxMin) || notNull(cpuMinmaxMax)}
                            label="Frequency Limits"
                            description="Set bounds on clock speed"
                            onChange={(value) => cpuDispatch(["freqToggleAdvanced", value])}
                        />
                        {notNull(cpuMinmaxMin) && (
                            <SliderRow
                                label="Minimum (MHz)"
                                value={cpuMinmaxMin}
                                max={cpu0Limits.clock_min_limits.max}
                                min={cpu0Limits.clock_min_limits.min}
                                step={cpu0Limits.clock_step}
                                showValue={true}
                                disabled={isNull(cpuMinmaxMin)}
                                onChange={(freq) => cpuDispatch(["minFreqAdvanced", freq])}
                            />
                        )}
                        {notNull(cpuMinmaxMax) && (
                            <SliderRow
                                label="Maximum (MHz)"
                                value={cpuMinmaxMax}
                                max={cpu0Limits.clock_max_limits.max}
                                min={cpu0Limits.clock_max_limits.min}
                                step={cpu0Limits.clock_step}
                                showValue={true}
                                disabled={isNull(cpuMinmaxMax)}
                                onChange={(freq) => cpuDispatch(["maxFreqAdvanced", freq])}
                            />
                        )}

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
                    </>
                )}
            </PanelSection>
            <PanelSection title="GPU">
                <ToggleRow
                    checked={notNull(gpuState.GPU_slowPPT) || notNull(gpuState.GPU_fastPPT)}
                    label="PowerPlay Limits"
                    description="Override APU TDP settings"
                    onChange={(toggle: boolean) => gpuDispatch(["pptToggle", toggle])}
                />
                {notNull(gpuState.GPU_slowPPT) && (
                    <SliderRow
                        label="SlowPPT (W)"
                        value={gpuState.GPU_slowPPT}
                        max={SETTINGS_LIMITS.gpu.slow_ppt_limits.max}
                        min={SETTINGS_LIMITS.gpu.slow_ppt_limits.min}
                        step={SETTINGS_LIMITS.gpu.ppt_step}
                        showValue={true}
                        disabled={isNull(gpuState.GPU_slowPPT)}
                        onChange={(ppt) => gpuDispatch(["slowPPT", ppt])}
                    />
                )}
                {notNull(gpuState.GPU_fastPPT) && (
                    <SliderRow
                        label="FastPPT (W)"
                        value={gpuState.GPU_fastPPT}
                        max={SETTINGS_LIMITS.gpu.fast_ppt_limits.max}
                        min={SETTINGS_LIMITS.gpu.fast_ppt_limits.min}
                        step={SETTINGS_LIMITS.gpu.ppt_step}
                        showValue={true}
                        disabled={isNull(gpuState.GPU_fastPPT)}
                        onChange={(ppt) => gpuDispatch(["fastPPT", ppt])}
                    />
                )}
                <ToggleRow
                    checked={notNull(gpuState.GPU_min_clock) || notNull(gpuState.GPU_max_clock)}
                    label="Frequency Limits"
                    description="Override bounds on gpu clock"
                    onChange={(toggle) => gpuDispatch(["freqToggle", toggle])}
                />
                {notNull(gpuState.GPU_min_clock) && (
                    <SliderRow
                        label="Minimum (MHz)"
                        value={gpuState.GPU_min_clock}
                        max={SETTINGS_LIMITS.gpu.clock_min_limits.max}
                        min={SETTINGS_LIMITS.gpu.clock_min_limits.min}
                        step={SETTINGS_LIMITS.gpu.clock_step}
                        showValue={true}
                        disabled={isNull(gpuState.GPU_min_clock)}
                        onChange={(val) => gpuDispatch(["minClock", val])}
                    />
                )}
                {notNull(gpuState.GPU_max_clock) && (
                    <SliderRow
                        label="Maximum (MHz)"
                        value={gpuState.GPU_max_clock}
                        max={SETTINGS_LIMITS.gpu.clock_max_limits.max}
                        min={SETTINGS_LIMITS.gpu.clock_max_limits.min}
                        step={SETTINGS_LIMITS.gpu.clock_step}
                        showValue={true}
                        disabled={isNull(gpuState.GPU_max_clock)}
                        onChange={(val) => gpuDispatch(["maxClock", val])}
                    />
                )}
                <ToggleRow
                    checked={gpuState.GPU_slow_memory}
                    label="Downclock Memory"
                    description="Force RAM into low-power mode"
                    onChange={(value) => gpuDispatch(["slowMemory", value])}
                />
            </PanelSection>
            <PanelSection title="Battery">
                <FieldRow label="Now (Charge)" onClick={() => setEggCount(eggCount + 1)}>
                    {chargeNow}
                </FieldRow>
                <FieldRow label="Max (Design)" onClick={() => setEggCount(eggCount + 1)}>
                    {chargeMax}
                </FieldRow>
                <>
                    <ToggleRow
                        checked={notNull(batteryState.BATTERY_charge_rate)}
                        label="Charge Current Limits"
                        description="Control battery charge rate when awake"
                        onChange={(toggle) => batteryDispatch(["chargeRateToggle", toggle])}
                    />
                    <SliderRow
                        label="Maximum (mA)"
                        value={batteryState.BATTERY_charge_rate ?? 0}
                        max={SETTINGS_LIMITS.battery.charge_current.max}
                        min={SETTINGS_LIMITS.battery.charge_current.min}
                        step={SETTINGS_LIMITS.battery.charge_current_step}
                        showValue={true}
                        disabled={isNull(batteryState.BATTERY_charge_rate)}
                        onChange={(val) => batteryDispatch(["chargeRate", val])}
                    />
                </>
                <ToggleRow
                    checked={notNull(batteryState.BATTERY_charge_mode)}
                    label="Charge Mode"
                    description="Force battery charge mode"
                    onChange={(toggle) =>
                        batteryDispatch(["chargeModeToggle", { toggle, value: chargeModeOptions[0].data }])
                    }
                />
                <FieldRow label="Mode">
                    <Dropdown
                        menuLabel="Charge Mode"
                        rgOptions={chargeModeOptions}
                        selectedOption={chargeModeOptions.find((val) => val.data === batteryState.BATTERY_charge_mode)}
                        strDefaultLabel={batteryState.BATTERY_charge_mode ?? "BATTERY_charge_mode"}
                        onChange={(elem) => batteryDispatch(["chargeMode", elem.data])}
                    />
                </FieldRow>
                <FieldRow label="Current" onClick={() => setEggCount(eggCount + 1)}>
                    {batteryState.BATTERY_current_now} mA
                </FieldRow>
            </PanelSection>
            <PanelSection title="Miscellaneous">
                <ToggleRow
                    checked={generalState.GENERAL_persistent}
                    label="Persistent"
                    description="Save profile and load it next time"
                    onChange={(persist) => generalDispatch(["setPersistent", persist])}
                />
                <FieldRow label="Profile">{generalState.GENERAL_name || "NULL"}</FieldRow>
                {/* Version Info */}
            </PanelSection>
            <PanelSection title={isNerd ? "Ha! Nerd" : "Debug"}>
                <FieldRow
                    label={isNerd ? "PowerTools" : "Native"}
                    onClick={() => {
                        // you know you're bored and/or conceited when you spend time adding an easter egg
                        // that just sends people to your own project's repo
                        setEggCount(eggCount + 1);
                        if (isNerd) Router.NavigateToExternalWeb("https://github.com/NGnius/PowerTools");
                    }}
                >
                    {isNerd ? "by NGnius" : generalState.V_INFO}
                </FieldRow>
                <FieldRow label="Framework">{targetUsdpl()}</FieldRow>
                <FieldRow
                    label="USDPL"
                    onClick={() => {
                        // you know you're bored and/or conceited when you spend time adding an easter egg
                        // that just sends people to your own project's repo
                        setEggCount(eggCount + 1);
                        if (isNerd) Router.NavigateToExternalWeb("https://github.com/NGnius/usdpl-rs");
                    }}
                >
                    v{versionUsdpl()}
                </FieldRow>
                {isNerd && (
                    <ButtonRow bottomSeparator="none" onClick={() => generalDispatch(["idk"])}>
                        ???
                    </ButtonRow>
                )}
                <ButtonRow
                    bottomSeparator="none"
                    onClick={() =>
                        generalDispatch([
                            "loadSystemDefaults",
                            () => reload({ usdplReady: true, fullReload: true, smtAllowed }),
                        ])
                    }
                >
                    Defaults
                </ButtonRow>
            </PanelSection>
        </>
    );
};
