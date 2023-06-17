import { Fragment } from "react";
import {Component} from "react";
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
    CHARGE_DESIGN_BATT,
    CHARGE_FULL_BATT,
    CHARGE_NOW_BATT,
    CHARGE_RATE_BATT,
    CHARGE_MODE_BATT,
    CURRENT_BATT,
    CHARGE_LIMIT_BATT,
    CHARGE_POWER_BATT,
} from "../consts";
import { set_value, get_value} from "usdpl-front";

export class Battery extends Component<backend.IdcProps> {
    constructor(props: backend.IdcProps) {
        super(props);
        this.state = {
            reloadThingy: "/shrug",
        };
    }

    render() {
        const reloadGUI = (x: string) => this.setState({reloadThingy: x});
        const chargeModeOptions: SingleDropdownOption[] = (get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_modes.map((elem) => {return {
            data: elem,
            label: <span>{elem}</span>,
        };});
        return (<Fragment>
            {/* Battery */}
      <div className={staticClasses.PanelSectionTitle}>
        {tr("Battery")}
      </div>
      {get_value(CHARGE_NOW_BATT) != null && get_value(CHARGE_FULL_BATT) != null && <PanelSectionRow>
        <Field
          label={tr("Now (Charge)")}>
          {get_value(CHARGE_NOW_BATT).toFixed(1)} Wh ({(100 * get_value(CHARGE_NOW_BATT) / get_value(CHARGE_FULL_BATT)).toFixed(1)}%)
        </Field>
      </PanelSectionRow>}
      {get_value(CHARGE_FULL_BATT) != null && get_value(CHARGE_DESIGN_BATT) != null && <PanelSectionRow>
        <Field
          label={tr("Max (Design)")}>
          {get_value(CHARGE_FULL_BATT).toFixed(1)} Wh ({(100 * get_value(CHARGE_FULL_BATT) / get_value(CHARGE_DESIGN_BATT)).toFixed(1)}%)
        </Field>
      </PanelSectionRow>}
      {get_value(CHARGE_POWER_BATT) != null && get_value(CHARGE_POWER_BATT) > 0 && <PanelSectionRow>
        <Field
          label={tr("Charge Power")}>
          {get_value(CHARGE_POWER_BATT).toFixed(2)} W
        </Field>
      </PanelSectionRow>}
      {get_value(CURRENT_BATT) != null && <PanelSectionRow>
        <Field
          label={tr("Current")}>
          {get_value(CURRENT_BATT)} mA
        </Field>
      </PanelSectionRow>}
      {(get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_current != null && <PanelSectionRow>
        <ToggleField
          checked={get_value(CHARGE_RATE_BATT) != null}
          label={tr("Charge Current Limits")}
          description={tr("Control battery charge rate when awake")}
          onChange={(value: boolean) => {
            if (value) {
              set_value(CHARGE_RATE_BATT, (get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_current!.max);
              reloadGUI("BATTChargeRateToggle");
            } else {
              set_value(CHARGE_RATE_BATT, null);
              backend.resolve(backend.unsetBatteryChargeRate(), (_: any[]) => {
                reloadGUI("BATTUnsetChargeRate");
              });
            }
          }}
        />
        { get_value(CHARGE_RATE_BATT) != null && <SliderField
          label={tr("Maximum (mA)")}
          value={get_value(CHARGE_RATE_BATT)}
          max={(get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_current!.max}
          min={(get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_current!.min}
          step={(get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_current_step}
          showValue={true}
          disabled={get_value(CHARGE_RATE_BATT) == null}
          onChange={(val: number) => {
            backend.log(backend.LogLevel.Debug, "Charge rate is now " + val.toString());
            const rateNow = get_value(CHARGE_RATE_BATT);
            if (val != rateNow) {
              backend.resolve(backend.setBatteryChargeRate(val),
                              (rate: number) => {
                set_value(CHARGE_RATE_BATT, rate);
                reloadGUI("BATTChargeRate");
              });
            }
          }}
        />}
      </PanelSectionRow>}
      {chargeModeOptions.length != 0 && <PanelSectionRow>
        <ToggleField
          checked={get_value(CHARGE_MODE_BATT) != null}
          label={tr("Charge Mode")}
          description={tr("Force battery charge mode")}
          onChange={(value: boolean) => {
            if (value) {
              set_value(CHARGE_MODE_BATT, chargeModeOptions[0].data as string);
              reloadGUI("BATTChargeModeToggle");
            } else {
              set_value(CHARGE_MODE_BATT, null);
              backend.resolve(backend.unsetBatteryChargeMode(), (_: any[]) => {
                reloadGUI("BATTUnsetChargeMode");
              });
            }
          }}
        />
        {get_value(CHARGE_MODE_BATT) != null && <Field
          label={tr("Mode")}
        >
          <Dropdown
            menuLabel={tr("Charge Mode")}
            rgOptions={chargeModeOptions}
            selectedOption={chargeModeOptions.find((val: SingleDropdownOption, _index, _arr) => {
              return val.data == get_value(CHARGE_MODE_BATT);
            })}
            strDefaultLabel={get_value(CHARGE_MODE_BATT)}
            onChange={(elem: SingleDropdownOption) => {
              backend.log(backend.LogLevel.Debug, "Charge mode dropdown selected " + elem.data.toString());
              backend.resolve(backend.setBatteryChargeMode(elem.data as string), (mode: string) => {
                set_value(CHARGE_MODE_BATT, mode);
                reloadGUI("BATTChargeMode");
              });
            }}
          />
        </Field>}
      </PanelSectionRow>}
      {(get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_limit != null && <PanelSectionRow>
        <ToggleField
          checked={get_value(CHARGE_LIMIT_BATT) != null}
          label={tr("Charge Limit")}
          description={tr("Limit battery charge when awake")}
          onChange={(value: boolean) => {
            if (value) {
              set_value(CHARGE_LIMIT_BATT, (get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_limit!.max);
              reloadGUI("BATTChargeLimitToggle");
            } else {
              set_value(CHARGE_LIMIT_BATT, null);
              backend.resolve(backend.unsetBatteryChargeLimit(), (_: any[]) => {
                reloadGUI("BATTUnsetChargeRate");
              });
            }
          }}
        />
        { get_value(CHARGE_LIMIT_BATT) != null && <SliderField
          label={tr("Maximum (%)")}
          value={get_value(CHARGE_LIMIT_BATT)}
          max={(get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_limit!.max}
          min={(get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_limit!.min}
          step={(get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_limit_step}
          showValue={true}
          disabled={get_value(CHARGE_LIMIT_BATT) == null}
          onChange={(val: number) => {
            backend.log(backend.LogLevel.Debug, "Charge limit is now " + val.toString());
            const rateNow = get_value(CHARGE_LIMIT_BATT);
            if (val != rateNow) {
              backend.resolve(backend.setBatteryChargeLimit(val),
                              (rate: number) => {
                set_value(CHARGE_LIMIT_BATT, rate);
                reloadGUI("BATTChargeLimit");
              });
            }
          }}
        />}
      </PanelSectionRow>}
            </Fragment>);
    }
}
