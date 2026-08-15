import React, { Suspense } from 'react';

interface OptionalRendererProps {
    /** Peer package that must be installed for the wrapped renderer to load. */
    peerDependency: string;
    /** Human-readable name of the capability, used in the fallback message. */
    capability: string;
    /** Placeholder rendered while the lazy chunk is still loading. */
    loading: React.ReactNode;
    children: React.ReactNode;
}

interface OptionalRendererState {
    failed: boolean;
}

/**
 * `three` and `recharts` are optional peer dependencies: hosts that never render
 * 3D scenes or charts should not pay to install them. Their renderers are loaded
 * from separate chunks, so a missing peer surfaces as a rejected dynamic import.
 * Without a boundary that rejection would propagate to the host application, so
 * this component degrades to an accessible explanation instead of unmounting the
 * surrounding surface.
 */
export class OptionalRenderer extends React.Component<OptionalRendererProps, OptionalRendererState> {
    state: OptionalRendererState = { failed: false };

    static getDerivedStateFromError(): OptionalRendererState {
        return { failed: true };
    }

    componentDidCatch(error: unknown) {
        if (typeof console !== 'undefined') {
            console.error(
                `[adk-ui-react] ${this.props.capability} could not be rendered. ` +
                    `Install the optional peer dependency "${this.props.peerDependency}" to enable it.`,
                error,
            );
        }
    }

    render() {
        if (this.state.failed) {
            return (
                <div className="adk-ui-optional-renderer" role="note">
                    <p className="adk-ui-optional-renderer__title">{this.props.capability} is unavailable</p>
                    <p className="adk-ui-optional-renderer__hint">
                        Install the optional peer dependency <code>{this.props.peerDependency}</code> to enable it.
                    </p>
                </div>
            );
        }

        return <Suspense fallback={this.props.loading}>{this.props.children}</Suspense>;
    }
}
