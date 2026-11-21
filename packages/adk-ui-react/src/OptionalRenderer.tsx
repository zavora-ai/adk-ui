import React, { Suspense } from 'react';

interface OptionalRendererProps {
    /** Peer package that must be installed for the wrapped renderer to load. */
    peerDependency: string;
    /** Human-readable name of the capability, used in the console message. */
    capability: string;
    /**
     * What is drawn while the lazy chunk loads, and instead of it when the peer is missing:
     * the ADK-Rust home's own view of the component, which shows the same checked content.
     */
    fallback: React.ReactNode;
    children: React.ReactNode;
}

interface OptionalRendererState {
    failed: boolean;
}

/**
 * `recharts` is an optional peer dependency: a host that never draws rich charts need not
 * install it. Its renderer loads from a separate chunk, so a missing peer surfaces as a
 * rejected dynamic import. This boundary keeps that rejection from taking down the drawing and
 * draws the fallback instead.
 */
export class OptionalRenderer extends React.Component<OptionalRendererProps, OptionalRendererState> {
    state: OptionalRendererState = { failed: false };

    static getDerivedStateFromError(): OptionalRendererState {
        return { failed: true };
    }

    componentDidCatch(error: unknown) {
        if (typeof console !== 'undefined') {
            console.warn(
                `[adk-ui-react] ${this.props.capability} fell back to the home's view. ` +
                    `Install the optional peer dependency "${this.props.peerDependency}" to draw it richly.`,
                error,
            );
        }
    }

    render() {
        if (this.state.failed) return <>{this.props.fallback}</>;
        return <Suspense fallback={this.props.fallback}>{this.props.children}</Suspense>;
    }
}
