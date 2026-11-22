// @zavora-ai/adk-ui-react 3: renderers of adk-ui's extension catalogs for
// @zavora-ai/adk-surface-react's DrawnView, and the same components for a host's own data.
import './styles.css';

export { INSIGHT_CATALOG_ID, SPATIAL_CATALOG_ID, STUDIO_CATALOG_ID } from './catalogs';
export { adkUiRenderers, insightRenderers, spatialRenderers, studioRenderers } from './registry';
export {
    AppShell,
    Badge,
    Chart,
    KeyValueList,
    Metric,
    PageView,
    ProgressBar,
    RegionView,
    Sparkline,
} from './components';
export type { ShellPage } from './components';
export { KitProvider, isApprovedKit, isKitColour, kitA2uiTheme, kitStyleVariables, useKit } from './kit';
export type { KitManifest } from './kit';
export { OptionalRenderer } from './OptionalRenderer';
export { compact, formatNumber, formatValue } from './format';
export type * from './types';
