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
import {
    LIMITS_INFO,
    CHARGE_DESIGN_BATT,
    CHARGE_FULL_BATT,
    CHARGE_NOW_BATT,
    CHARGE_RATE_BATT,
    CHARGE_MODE_BATT,
    CURRENT_BATT,
} from "../consts";
import { set_value, get_value} from "usdpl-front";

export class Battery extends Component<{}> {
    constructor(props: {}) {
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
        Battery
      </div>
      {get_value(CHARGE_NOW_BATT) != null && get_value(CHARGE_FULL_BATT) != null && <PanelSectionRow>
        <Field
          label="Now (Charge)">
          {get_value(CHARGE_NOW_BATT).toFixed(1)} Wh ({(100 * get_value(CHARGE_NOW_BATT) / get_value(CHARGE_FULL_BATT)).toFixed(1)}%)
        </Field>
      </PanelSectionRow>}
      {get_value(CHARGE_FULL_BATT) != null && get_value(CHARGE_DESIGN_BATT) != null && <PanelSectionRow>
        <Field
          label="Max (Design)">
          {get_value(CHARGE_FULL_BATT).toFixed(1)} Wh ({(100 * get_value(CHARGE_FULL_BATT) / get_value(CHARGE_DESIGN_BATT)).toFixed(1)}%)
        </Field>
      </PanelSectionRow>}
      {(get_value(LIMITS_INFO) as backend.SettingsLimits).battery.charge_current != null && <PanelSectionRow>
        <ToggleField
          checked={get_value(CHARGE_RATE_BATT) != null}
          label="Charge Current Limits"
          description="Control battery charge rate when awake"
          onChange={(value: boolean) => {
            if (value) {
              set_value(CHARGE_RATE_BATT, 2500);
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
          label="Maximum (mA)"
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
          label="Charge Mode"
          description="Force battery charge mode"
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
          label="Mode"
        >
          <Dropdown
            menuLabel="Charge Mode"
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
      <PanelSectionRow>
        <Field
          label="Current">
          {get_value(CURRENT_BATT)} mA
        </Field>
      </PanelSectionRow>
            </Fragment>);
    }
}
