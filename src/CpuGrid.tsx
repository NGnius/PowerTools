import { VFC } from "react";
import { FieldRow } from "./Fields";

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
