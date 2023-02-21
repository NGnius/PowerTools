import { Fragment } from "react";
import {Component} from "react";
import {
  ToggleField,
  SliderField,
  PanelSectionRow,
  staticClasses,
} from "decky-frontend-lib";
import * as backend from "../backend";
import { tr } from "usdpl-front";
import {
    LIMITS_INFO,
    SLOW_PPT_GPU,
    FAST_PPT_GPU,
    CLOCK_MIN_GPU,
    CLOCK_MAX_GPU,
    SLOW_MEMORY_GPU,
} from "../consts";
import { set_value, get_value} from "usdpl-front";

export class Gpu extends Component<backend.IdcProps> {
    constructor(props: backend.IdcProps) {
        super(props);
        this.state = {
            reloadThingy: "/shrug",
        };
    }

    render() {
        const reloadGUI = (x: string) => this.setState({reloadThingy: x});
        return (<Fragment>
                {/* GPU */}
            <div className={staticClasses.PanelSectionTitle}>
                {tr("GPU")}
            </div>
            { ((get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.fast_ppt_limits != null ||(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.slow_ppt_limits != null) && <PanelSectionRow>
                <ToggleField
                checked={get_value(SLOW_PPT_GPU) != null || get_value(FAST_PPT_GPU) != null}
                label={tr("PowerPlay Limits")}
                description={tr("Override APU TDP settings")}
                onChange={(value: boolean) => {
                    if (value) {
                        if ((get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.slow_ppt_limits != null) {
                            set_value(SLOW_PPT_GPU, (get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.slow_ppt_limits!.max);
                        }

                        if ((get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.fast_ppt_limits != null) {
                            set_value(FAST_PPT_GPU, (get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.fast_ppt_limits!.max);
                        }
                        reloadGUI("GPUPPTToggle");
                    } else {
                        set_value(SLOW_PPT_GPU, null);
                        set_value(FAST_PPT_GPU, null);
                        backend.resolve(backend.unsetGpuPpt(), (_: any[]) => {
                            reloadGUI("GPUUnsetPPT");
                        });
                    }
                }}
                />
            </PanelSectionRow>}
            <PanelSectionRow>
                { get_value(SLOW_PPT_GPU) != null && <SliderField
                label={tr("SlowPPT (W)")}
                value={get_value(SLOW_PPT_GPU)}
                max={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.slow_ppt_limits!.max}
                min={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.slow_ppt_limits!.min}
                step={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.ppt_step}
                showValue={true}
                disabled={get_value(SLOW_PPT_GPU) == null}
                onChange={(ppt: number) => {
                    backend.log(backend.LogLevel.Debug, "SlowPPT is now " + ppt.toString());
                    const pptNow = get_value(SLOW_PPT_GPU);
                    const realPpt = ppt;
                    if (realPpt != pptNow) {
                    backend.resolve(backend.setGpuPpt(get_value(FAST_PPT_GPU), realPpt),
                                    (limits: number[]) => {
                        set_value(FAST_PPT_GPU, limits[0]);
                        set_value(SLOW_PPT_GPU, limits[1]);
                        reloadGUI("GPUSlowPPT");
                    });
                    }
                }}
                />}
            </PanelSectionRow>
            <PanelSectionRow>
                {get_value(FAST_PPT_GPU) != null && <SliderField
                label={tr("FastPPT (W)")}
                value={get_value(FAST_PPT_GPU)}
                max={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.fast_ppt_limits!.max}
                min={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.fast_ppt_limits!.min}
                step={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.ppt_step}
                showValue={true}
                disabled={get_value(FAST_PPT_GPU) == null}
                onChange={(ppt: number) => {
                    backend.log(backend.LogLevel.Debug, "FastPPT is now " + ppt.toString());
                    const pptNow = get_value(FAST_PPT_GPU);
                    const realPpt = ppt;
                    if (realPpt != pptNow) {
                    backend.resolve(backend.setGpuPpt(realPpt, get_value(SLOW_PPT_GPU)),
                                    (limits: number[]) => {
                        set_value(FAST_PPT_GPU, limits[0]);
                        set_value(SLOW_PPT_GPU, limits[1]);
                        reloadGUI("GPUFastPPT");
                    });
                    }
                }}
                />}
            </PanelSectionRow>
            {((get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.clock_min_limits != null || (get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.clock_max_limits != null) && <PanelSectionRow>
                <ToggleField
                checked={get_value(CLOCK_MIN_GPU) != null || get_value(CLOCK_MAX_GPU) != null}
                label={tr("Frequency Limits")}
                description={tr("Set bounds on clock speed")}
                onChange={(value: boolean) => {
                    if (value) {
                        let clock_min_limits = (get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.clock_min_limits;
                        let clock_max_limits = (get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.clock_max_limits;
                        if (clock_min_limits != null) {
                            set_value(CLOCK_MIN_GPU, clock_min_limits.min);
                        }
                        if (clock_max_limits != null) {
                            set_value(CLOCK_MAX_GPU, clock_max_limits.max);
                        }
                        reloadGUI("GPUFreqToggle");
                    } else {
                        set_value(CLOCK_MIN_GPU, null);
                        set_value(CLOCK_MAX_GPU, null);
                        backend.resolve(backend.unsetGpuClockLimits(), (_: any[]) => {
                            reloadGUI("GPUUnsetFreq");
                        });
                    }
                }}
                />
            </PanelSectionRow>}
            <PanelSectionRow>
                { get_value(CLOCK_MIN_GPU) != null && <SliderField
                label={tr("Minimum (MHz)")}
                value={get_value(CLOCK_MIN_GPU)}
                max={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.clock_min_limits!.max}
                min={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.clock_min_limits!.min}
                step={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.clock_step}
                showValue={true}
                disabled={get_value(CLOCK_MIN_GPU) == null}
                onChange={(val: number) => {
                    backend.log(backend.LogLevel.Debug, "GPU Clock Min is now " + val.toString());
                    const valNow = get_value(CLOCK_MIN_GPU);
                    const maxNow = get_value(CLOCK_MAX_GPU);
                    if (val != valNow && ((maxNow != null && val <= maxNow) || maxNow == null)) {
                        backend.resolve(backend.setGpuClockLimits(val, get_value(CLOCK_MAX_GPU)),
                                        (limits: number[]) => {
                            set_value(CLOCK_MIN_GPU, limits[0]);
                            set_value(CLOCK_MAX_GPU, limits[1]);
                            reloadGUI("GPUMinClock");
                        });
                    }
                }}
                />}
            </PanelSectionRow>
            <PanelSectionRow>
                {get_value(CLOCK_MAX_GPU) != null && <SliderField
                label={tr("Maximum (MHz)")}
                value={get_value(CLOCK_MAX_GPU)}
                max={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.clock_max_limits!.max}
                min={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.clock_max_limits!.min}
                step={(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.clock_step}
                showValue={true}
                disabled={get_value(CLOCK_MAX_GPU) == null}
                onChange={(val: number) => {
                    backend.log(backend.LogLevel.Debug, "GPU Clock Max is now " + val.toString());
                    const valNow = get_value(CLOCK_MAX_GPU);
                    const minNow = get_value(CLOCK_MIN_GPU);
                    if (val != valNow && ((minNow != null && val >= minNow) || minNow == null)) {
                        backend.resolve(backend.setGpuClockLimits(get_value(CLOCK_MIN_GPU), val),
                                        (limits: number[]) => {
                            set_value(CLOCK_MIN_GPU, limits[0]);
                            set_value(CLOCK_MAX_GPU, limits[1]);
                            reloadGUI("GPUMaxClock");
                        });
                    }
                }}
                />}
            </PanelSectionRow>
            {(get_value(LIMITS_INFO) as backend.SettingsLimits).gpu.memory_control_capable && <PanelSectionRow>
                <ToggleField
                checked={get_value(SLOW_MEMORY_GPU)}
                label={tr("Downclock Memory")}
                description={tr("Force RAM into low-power mode")}
                onChange={(value: boolean) => {
                    backend.resolve(backend.setGpuSlowMemory(value), (val: boolean) => {
                        set_value(SLOW_MEMORY_GPU, val);
                        reloadGUI("GPUSlowMemory");
                    })
                }}
                />
            </PanelSectionRow>}
            </Fragment>);
    }
}
