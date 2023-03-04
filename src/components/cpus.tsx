import { Fragment } from "react";
import { Component } from "react";
import {
  ToggleField,
  SliderField,
  Field,
  SingleDropdownOption,
  Dropdown,
  PanelSectionRow,
  staticClasses,
} from "decky-frontend-lib";
import * as backend from "../backend";
import { tr } from "usdpl-front";
import {
    LIMITS_INFO,
    SMT_CPU,
    CLOCK_MAX_CPU,
    CLOCK_MIN_CPU,
    CLOCK_MIN_MAX_CPU,
    ONLINE_CPUS,
    ONLINE_STATUS_CPUS,
    GOVERNOR_CPU,
} from "../consts";
import { set_value, get_value } from "usdpl-front";

interface CpuState {
    reloadThingy: string;
}

let advancedMode = false;
let advancedCpu = 1;

export class Cpus extends Component<backend.IdcProps, CpuState> {
    constructor(props: backend.IdcProps) {
        super(props);
        this.state = {
            reloadThingy: "/shrug",
        };
    }

    render() {
        const reloadGUI = (x: string) => this.setState((_state) => {
            return {
                reloadThingy: x,
            };
        });

        const total_cpus = (get_value(LIMITS_INFO) as backend.SettingsLimits | null)?.cpu.count ?? 8;
        const advancedCpuIndex = advancedCpu - 1;
        const smtAllowed = (get_value(LIMITS_INFO) as backend.SettingsLimits | null)?.cpu.smt_capable ?? true;

        const governorOptions: SingleDropdownOption[] = (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].governors.map((elem) => {return {
            data: elem,
            label: <span>{elem}</span>,
        };});

        const governorGlobalOptions: SingleDropdownOption[] = (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.governors.map((elem) => {return {
            data: elem,
            label: <span>{elem}</span>,
        };});

        return (<Fragment>
            {/* CPU */}
                <div className={staticClasses.PanelSectionTitle}>
                    {tr("CPU")}
                </div>
                <PanelSectionRow>
                    <ToggleField
                    checked={advancedMode}
                    label={tr("Advanced")}
                    description={tr("Enables per-thread configuration")}
                    onChange={(advanced: boolean) => {
                        advancedMode = advanced;
                        this.setState((state) => {
                            return {
                                reloadThingy: state.reloadThingy,
                            };
                        });
                    }}
                    />
                </PanelSectionRow>
                {/* CPU plebeian mode */}
                {!advancedMode && smtAllowed && <PanelSectionRow>
                    <ToggleField
                    checked={get_value(SMT_CPU)}
                    label={tr("SMT")}
                    description={tr("Enables odd-numbered CPUs")}
                    onChange={(smt: boolean) => {
                        backend.log(backend.LogLevel.Debug, "SMT is now " + smt.toString());
                        //const cpus = get_value(ONLINE_CPUS);
                        const smtNow = smt && smtAllowed;
                        backend.resolve(backend.setCpuSmt(smtNow), (statii: boolean[]) => {
                        set_value(SMT_CPU, smtNow);
                        set_value(ONLINE_STATUS_CPUS, statii);
                        const count = countCpus(statii);
                        set_value(ONLINE_CPUS, count);
                        reloadGUI("SMT");
                        });
                    }}
                    />
                </PanelSectionRow>}
                {!advancedMode && <PanelSectionRow>
                    <SliderField
                    label={tr("Threads")}
                    value={get_value(ONLINE_CPUS)}
                    step={1}
                    max={(get_value(SMT_CPU) || !smtAllowed) ? total_cpus : total_cpus/2}
                    min={1}
                    showValue={true}
                    onChange={(cpus: number) => {
                        backend.log(backend.LogLevel.Debug, "CPU slider is now " + cpus.toString());
                        const onlines = get_value(ONLINE_CPUS);
                        if (cpus != onlines) {
                        set_value(ONLINE_CPUS, cpus);
                        const smtNow = get_value(SMT_CPU);
                        let onlines: boolean[] = [];
                        for (let i = 0; i < total_cpus; i++) {
                            const online = smtNow? i < cpus : (i % 2 == 0) && (i < cpus * 2);
                            onlines.push(online);
                        }
                        backend.resolve(backend.setCpuOnlines(onlines), (statii: boolean[]) => {
                            set_value(ONLINE_STATUS_CPUS, statii);
                            const count = countCpus(statii);
                            set_value(ONLINE_CPUS, count);
                            reloadGUI("CPUs");
                        });
                        reloadGUI("CPUsImmediate");
                        }
                    }}
                    />
                </PanelSectionRow>}
                {!advancedMode && <PanelSectionRow>
                    <ToggleField
                    checked={get_value(CLOCK_MIN_CPU) != null || get_value(CLOCK_MAX_CPU) != null}
                    label={tr("Frequency Limits")}
                    description={tr("Set bounds on clock speed")}
                    onChange={(value: boolean) => {
                        if (value) {
                            if ((get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_min_limits != null) {
                                set_value(CLOCK_MIN_CPU, (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_min_limits!.min);
                            }
                            if ((get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_max_limits != null) {
                                set_value(CLOCK_MAX_CPU, (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_max_limits!.max);
                            }
                            syncPlebClockToAdvanced();
                            reloadGUI("CPUFreqToggle");
                        } else {
                            set_value(CLOCK_MIN_CPU, null);
                            set_value(CLOCK_MAX_CPU, null);
                            for (let i = 0; i < total_cpus; i++) {
                                backend.resolve(backend.unsetCpuClockLimits(i), (_idc: any[]) => {});
                            }
                            backend.resolve(backend.waitForComplete(), (_: boolean) => {
                                reloadGUI("CPUUnsetFreq");
                            });
                            syncPlebClockToAdvanced();
                        }
                    }}
                    />
                </PanelSectionRow>}
                {!advancedMode && (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_min_limits != null && <PanelSectionRow>
                    {get_value(CLOCK_MIN_CPU) != null && <SliderField
                    label={tr("Minimum (MHz)")}
                    value={get_value(CLOCK_MIN_CPU)}
                    max={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_min_limits!.max}
                    min={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_min_limits!.min}
                    step={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_step}
                    showValue={true}
                    disabled={get_value(CLOCK_MIN_CPU) == null}
                    onChange={(freq: number) => {
                        backend.log(backend.LogLevel.Debug, "Min freq slider is now " + freq.toString());
                        const freqNow = get_value(CLOCK_MIN_CPU);
                        const maxNow = get_value(CLOCK_MAX_CPU);
                        if (freq != freqNow && ((maxNow != null && freq <= maxNow) || maxNow == null)) {
                            set_value(CLOCK_MIN_CPU, freq);
                            for (let i = 0; i < total_cpus; i++) {
                                backend.resolve(backend.setCpuClockLimits(i, freq, get_value(CLOCK_MAX_CPU)),
                                                (_limits: number[]) => {
                                //set_value(CLOCK_MIN_CPU, limits[0]);
                                //set_value(CLOCK_MAX_CPU, limits[1]);
                                syncPlebClockToAdvanced();
                                });
                            }
                            backend.resolve(backend.waitForComplete(), (_: boolean) => {
                                reloadGUI("CPUMinFreq");
                            });
                            reloadGUI("CPUMinFreqImmediate");
                        }
                    }}
                    />}
                </PanelSectionRow>}
                {!advancedMode && (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_max_limits != null && <PanelSectionRow>
                    {get_value(CLOCK_MAX_CPU) != null && <SliderField
                    label={tr("Maximum (MHz)")}
                    value={get_value(CLOCK_MAX_CPU)}
                    max={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_max_limits!.max}
                    min={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_max_limits!.min}
                    step={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[0].clock_step}
                    showValue={true}
                    disabled={get_value(CLOCK_MAX_CPU) == null}
                    onChange={(freq: number) => {
                        backend.log(backend.LogLevel.Debug, "Max freq slider is now " + freq.toString());
                        const freqNow = get_value(CLOCK_MAX_CPU);
                        const minNow = get_value(CLOCK_MIN_CPU);
                        if (freq != freqNow && ((minNow != null && freq >= minNow) || minNow == null)) {
                            set_value(CLOCK_MAX_CPU, freq);
                            for (let i = 0; i < total_cpus; i++) {
                                backend.resolve(backend.setCpuClockLimits(i, get_value(CLOCK_MIN_CPU), freq),
                                                (_limits: number[]) => {
                                //set_value(CLOCK_MIN_CPU, limits[0]);
                                //set_value(CLOCK_MAX_CPU, limits[1]);
                                syncPlebClockToAdvanced();
                                });
                            }
                            backend.resolve(backend.waitForComplete(), (_: boolean) => {
                                reloadGUI("CPUMaxFreq");
                            });
                            reloadGUI("CPUMaxFreqImmediate");
                        }
                    }}
                    />}
                </PanelSectionRow>}
                {!advancedMode && governorGlobalOptions.length != 0 && <PanelSectionRow>
                    <Field
                    label={tr("Governor")}
                    >
                    <Dropdown
                        menuLabel={tr("Governor")}
                        rgOptions={governorGlobalOptions}
                        selectedOption={governorGlobalOptions.find((val: SingleDropdownOption, _index, _arr) => {
                        backend.log(backend.LogLevel.Debug, "POWERTOOLS: array item " +  val.toString());
                        backend.log(backend.LogLevel.Debug, "POWERTOOLS: looking for data " + get_value(GOVERNOR_CPU)[0].toString());
                        return val.data == get_value(GOVERNOR_CPU)[0];
                        })}
                        strDefaultLabel={get_value(GOVERNOR_CPU)[0]}
                        onChange={(elem: SingleDropdownOption) => {
                            backend.log(backend.LogLevel.Debug, "Governor global dropdown selected " + elem.data.toString());
                            const governors = get_value(GOVERNOR_CPU);
                            for (let i = 0; i < total_cpus; i++) {
                                governors[i] = elem.data as string;
                                backend.resolve(backend.setCpuGovernor(i, governors[i]), (_: string) => {});
                            }
                            set_value(GOVERNOR_CPU, governors);
                            reloadGUI("CPUGlobalGovernor");
                        }}
                    />
                    </Field>
                </PanelSectionRow>}
                {/* CPU advanced mode */}
                {advancedMode && <PanelSectionRow>
                    <SliderField
                    label={tr("Selected CPU")}
                    value={advancedCpu}
                    step={1}
                    max={total_cpus}
                    min={1}
                    showValue={true}
                    onChange={(cpuNum: number) => {
                        advancedCpu = cpuNum;
                        this.setState((state) => {
                            return {
                                reloadThingy: state.reloadThingy,
                            };
                        });
                    }}
                    />
                </PanelSectionRow>}
                {advancedMode && <PanelSectionRow>
                    <ToggleField
                    checked={get_value(ONLINE_STATUS_CPUS)[advancedCpuIndex]}
                    label={tr("Online")}
                    description={tr("Allow the CPU thread to do work")}
                    onChange={(status: boolean) => {
                        backend.log(backend.LogLevel.Debug, "CPU " + advancedCpu.toString() + " is now " + status.toString());
                        if (!get_value(SMT_CPU)) {
                            backend.resolve(backend.setCpuSmt(true), (_newVal: boolean[]) => {
                                set_value(SMT_CPU, true);
                            });
                        }
                        backend.resolve(backend.setCpuOnline(advancedCpuIndex, status), (newVal: boolean) => {
                            const onlines = get_value(ONLINE_STATUS_CPUS);
                            onlines[advancedCpuIndex] = newVal;
                            set_value(ONLINE_STATUS_CPUS, onlines);
                        });
                    }}
                    />
                </PanelSectionRow>}
                {advancedMode && <PanelSectionRow>
                    <ToggleField
                    checked={get_value(CLOCK_MIN_MAX_CPU)[advancedCpuIndex].min != null || get_value(CLOCK_MIN_MAX_CPU)[advancedCpuIndex].max != null}
                    label={tr("Frequency Limits")}
                    description={tr("Set bounds on clock speed")}
                    onChange={(value: boolean) => {
                        if (value) {
                            const clocks = get_value(CLOCK_MIN_MAX_CPU) as MinMax[];
                            if ((get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_min_limits != null) {
                                clocks[advancedCpuIndex].min = (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_min_limits!.min;
                            }

                            if ((get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_max_limits != null) {
                                clocks[advancedCpuIndex].max = (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_max_limits!.max;
                            }
                            set_value(CLOCK_MIN_MAX_CPU, clocks);
                            reloadGUI("CPUFreqToggle");
                        } else {
                            const clocks = get_value(CLOCK_MIN_MAX_CPU) as MinMax[];
                            clocks[advancedCpuIndex].min = null;
                            clocks[advancedCpuIndex].max = null;
                            set_value(CLOCK_MIN_MAX_CPU, clocks);
                            backend.resolve(backend.unsetCpuClockLimits(advancedCpuIndex), (_idc: any[]) => {
                                reloadGUI("CPUUnsetFreq");
                            });
                        }
                    }}
                    />
                </PanelSectionRow>}
                {advancedMode && (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_min_limits != null && <PanelSectionRow>
                    {get_value(CLOCK_MIN_MAX_CPU)[advancedCpuIndex].min != null && <SliderField
                    label={tr("Minimum (MHz)")}
                    value={get_value(CLOCK_MIN_MAX_CPU)[advancedCpuIndex].min}
                    max={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_min_limits!.max}
                    min={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_min_limits!.min}
                    step={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_step}
                    showValue={true}
                    disabled={get_value(CLOCK_MIN_MAX_CPU)[advancedCpuIndex].min == null}
                    onChange={(freq: number) => {
                        backend.log(backend.LogLevel.Debug, "Min freq slider for " + advancedCpu.toString() + " is now " + freq.toString());
                        const freqNow = get_value(CLOCK_MIN_MAX_CPU)[advancedCpuIndex] as MinMax;
                        if (freq != freqNow.min && ((freqNow.max != null && freq <= freqNow.max) || freqNow.max == null)) {
                            backend.resolve(backend.setCpuClockLimits(advancedCpuIndex, freq, freqNow.max!),
                                                (limits: number[]) => {
                                const clocks = get_value(CLOCK_MIN_MAX_CPU) as MinMax[];
                                clocks[advancedCpuIndex].min = limits[0];
                                clocks[advancedCpuIndex].max = limits[1];
                                set_value(CLOCK_MIN_MAX_CPU, clocks);
                                reloadGUI("CPUMinFreq");
                            });
                        }
                    }}
                    />}
                </PanelSectionRow>}
                {advancedMode && (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_max_limits != null && <PanelSectionRow>
                    {get_value(CLOCK_MIN_MAX_CPU)[advancedCpuIndex].max != null && <SliderField
                    label={tr("Maximum (MHz)")}
                    value={get_value(CLOCK_MIN_MAX_CPU)[advancedCpuIndex].max}
                    max={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_max_limits!.max}
                    min={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_max_limits!.min}
                    step={(get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.cpus[advancedCpuIndex].clock_step}
                    showValue={true}
                    disabled={get_value(CLOCK_MIN_MAX_CPU)[advancedCpuIndex].max == null}
                    onChange={(freq: number) => {
                        backend.log(backend.LogLevel.Debug, "Max freq slider for " + advancedCpu.toString() + " is now " + freq.toString());
                        const freqNow = get_value(CLOCK_MIN_MAX_CPU)[advancedCpuIndex] as MinMax;
                        if (freq != freqNow.max && ((freqNow.min != null && freq >= freqNow.min) || freqNow.min == null)) {
                            backend.resolve(backend.setCpuClockLimits(advancedCpuIndex, freqNow.min!, freq),
                                            (limits: number[]) => {
                                const clocks = get_value(CLOCK_MIN_MAX_CPU) as MinMax[];
                                clocks[advancedCpuIndex].min = limits[0];
                                clocks[advancedCpuIndex].max = limits[1];
                                set_value(CLOCK_MIN_MAX_CPU, clocks);
                                reloadGUI("CPUMaxFreq");
                            });
                        }
                    }}
                    />}
                </PanelSectionRow>}
                {advancedMode && governorOptions.length != 0 && <PanelSectionRow>
                    <Field
                    label={tr("Governor")}
                    >
                    <Dropdown
                        menuLabel={tr("Governor")}
                        rgOptions={governorOptions}
                        selectedOption={governorOptions.find((val: SingleDropdownOption, _index, _arr) => {
                        backend.log(backend.LogLevel.Debug, "POWERTOOLS: array item " +  val.toString());
                        backend.log(backend.LogLevel.Debug, "POWERTOOLS: looking for data " + get_value(GOVERNOR_CPU)[advancedCpuIndex].toString());
                        return val.data == get_value(GOVERNOR_CPU)[advancedCpuIndex];
                        })}
                        strDefaultLabel={get_value(GOVERNOR_CPU)[advancedCpuIndex]}
                        onChange={(elem: SingleDropdownOption) => {
                        backend.log(backend.LogLevel.Debug, "Governor dropdown selected " + elem.data.toString());
                        backend.resolve(backend.setCpuGovernor(advancedCpuIndex, elem.data as string), (gov: string) => {
                            const governors = get_value(GOVERNOR_CPU);
                            governors[advancedCpuIndex] = gov;
                            set_value(GOVERNOR_CPU, governors);
                            reloadGUI("CPUGovernor");
                        });
                        }}
                    />
                    </Field>
                </PanelSectionRow>}
            </Fragment>);
    }
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

type MinMax = {
  min: number | null;
  max: number | null;
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
