/* eslint-disable @typescript-eslint/no-explicit-any */
import { Component } from "react";

export class ErrorBoundary extends Component<
    any,
    { error: Error | null; errorInfo: { componentStack: string[] } | null }
> {
    constructor(props: any) {
        super(props);
        this.state = { error: null, errorInfo: null };
    }
    componentDidCatch(error: Error, errorInfo: any): void {
        this.setState({
            error: error,
            errorInfo: errorInfo,
        });
    }
    render = () =>
        this.state.errorInfo ? (
            <>
                <h2>An Error Has Occurred</h2>
                <pre style={{ maxWidth: "100%", width: "100%", fontSize: "smaller", overflowWrap: "anywhere" }}>
                    {this.state.error && this.state.error.toString()}
                    <br />
                    {this.state.errorInfo.componentStack}
                </pre>
            </>
        ) : (
            this.props.children
        );
}
