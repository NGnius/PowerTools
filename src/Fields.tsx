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

const Core: VFC<{ online: boolean; index: string | number }> = ({ online, index }) => (
    <pre style={online ? {} : { filter: "invert(100%)" }}>
        <small>CPU</small> {index}
    </pre>
);

export const CpuGrid: VFC<{ cores: boolean[] }> = ({ cores }) => (
    <FieldRow label="CPU Status Overview">
        <div
            style={{
                alignItems: "center",
                display: "grid",
                gap: "1em",
                gridAutoRows: "1fr",
                gridTemplateColumns: "repeat(4, 1fr)",
                justifyContent: "center",
                margin: 0,
                padding: 0,
                width: "100%",
            }}
        >
            {cores.map((online, i) => (
                <Core key={`core-${i}`} index={i} online={online} />
            ))}
        </div>
    </FieldRow>
);
