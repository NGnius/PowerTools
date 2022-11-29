import {
    Field,
    ButtonItem,
    PanelSectionRow,
    SliderField,
    SliderFieldProps,
    ToggleField,
    ToggleFieldProps,
    ButtonItemProps,
} from "decky-frontend-lib";
import { ComponentProps, VFC, ReactNode } from "react";

export const FieldRow: VFC<ComponentProps<typeof Field>> = ({ children, ...props }) => (
    <PanelSectionRow>
        <Field focusable bottomSeparator="none" {...props}>
            {children}
        </Field>
    </PanelSectionRow>
);
export const SliderRow: VFC<SliderFieldProps> = (props) => (
    <PanelSectionRow>
        <SliderField bottomSeparator="none" {...props} />
    </PanelSectionRow>
);

export const ToggleRow: VFC<ToggleFieldProps> = (props) => (
    <PanelSectionRow>
        <ToggleField bottomSeparator="none" {...props} />
    </PanelSectionRow>
);
export const ButtonRow: VFC<ButtonItemProps & { children?: ReactNode }> = (props) => (
    <PanelSectionRow>
        <ButtonItem bottomSeparator="none" layout="below" {...props} />
    </PanelSectionRow>
);
