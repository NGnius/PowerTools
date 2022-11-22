import {
    PanelSectionRow,
    SliderField,
    SliderFieldProps,
    ToggleField,
    ToggleFieldProps,
    Field,
} from "decky-frontend-lib";
import { ComponentProps, VFC } from "react";

export const FieldRow: VFC<ComponentProps<typeof Field>> = ({ children, ...props }) => (
    <PanelSectionRow>
        <Field {...props}>{children}</Field>
    </PanelSectionRow>
);
export const SliderRow: VFC<SliderFieldProps> = (props) => (
    <PanelSectionRow>
        <SliderField {...props} />
    </PanelSectionRow>
);

export const ToggleRow: VFC<ToggleFieldProps> = (props) => (
    <PanelSectionRow>
        <ToggleField {...props} />
    </PanelSectionRow>
);
