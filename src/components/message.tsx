import { Component, Fragment } from "react";
import * as backend from "../backend";

import {
  Field,
  staticClasses,
  PanelSectionRow,
  ButtonItem,
  Navigation,
} from "decky-frontend-lib";

import { MESSAGE_LIST } from "../consts";

import { set_value, get_value, tr } from "usdpl-front";

export class DevMessages extends Component<backend.IdcProps> {
    constructor(props: backend.IdcProps) {
        super(props);
        this.state = {
            reloadThingy: "/shrug",
        };
    }

    render() {
        const reloadGUI = (x: string) => this.setState({reloadThingy: x});
        const messages: backend.Message[] = get_value(MESSAGE_LIST) as backend.Message[];
        if (messages.length != 0) {
            const message = messages[0];
            return (<Fragment>
                <div className={staticClasses.PanelSectionTitle}>
                    {message.title}
                </div>
                <PanelSectionRow>
                    <Field
                        onClick={()=> { if (message.url) { Navigation.NavigateToExternalWeb(message.url); } }}>
                        {message.body}
                    </Field>
                    <ButtonItem
                        layout="below"
                        onClick={(_: MouseEvent) => {
                            if (message.id) {
                                backend.dismissMessage(message.id);
                            }
                            messages.shift();
                            set_value(MESSAGE_LIST, messages);
                            reloadGUI("MessageDismissed");
                        }}
                        >
                        {tr("Dismiss")}
                    </ButtonItem>
                </PanelSectionRow>
                </Fragment>
            )
        } else {
            return <Fragment></Fragment>
        }
    }
}
