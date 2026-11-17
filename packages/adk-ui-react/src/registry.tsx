import { INSIGHT_CATALOG_ID, STUDIO_CATALOG_ID } from './catalogs';
import { AppShell, Badge, Chart, KeyValueList, Metric, PageView, ProgressBar, RegionView, Scene3D, Sparkline } from './components';
import type {
    AppProps,
    BadgeProps,
    ChartProps,
    KeyValueProps,
    MetricProps,
    PageProps,
    ProgressProps,
    RegionProps,
    Renderer,
    RendererProps,
    Renderers,
    Scene3DProps,
    SparklineProps,
} from './types';

/** What the ADK-Rust home shows for the component: the node's one child, drawn as the home does. */
function homeView({ node, renderChild }: RendererProps) {
    const view = node.children?.[0];
    return view ? renderChild(view) : null;
}

/** The server checked the properties against the catalog's schema before anything was drawn. */
const as = <T,>(props: Record<string, unknown>) => props as unknown as T;

const insight: Record<string, Renderer> = {
    Chart: (renderer) => <Chart {...as<ChartProps>(renderer.props)} fallback={homeView(renderer)} />,
    Sparkline: (renderer) => <Sparkline {...as<SparklineProps>(renderer.props)} />,
    Metric: (renderer) => <Metric {...as<MetricProps>(renderer.props)} />,
    KeyValue: (renderer) => <KeyValueList {...as<KeyValueProps>(renderer.props)} />,
    Progress: (renderer) => <ProgressBar {...as<ProgressProps>(renderer.props)} />,
    Badge: (renderer) => <Badge {...as<BadgeProps>(renderer.props)} />,
};

const studio: Record<string, Renderer> = {
    ...insight,
    App: ({ props, children, renderChild }) => {
        const app = as<AppProps>(props);
        const pages = app.pages.map((page, index) => ({
            id: page.id,
            title: page.title,
            content: children[index] ? renderChild(children[index]) : null,
        }));
        return <AppShell title={app.title} navigation={app.navigation} initialPage={app.initialPage} pages={pages} />;
    },
    Page: ({ props, children, renderChild }) => {
        const page = as<PageProps>(props);
        return (
            <PageView
                title={page.title}
                description={page.description}
                layout={page.layout}
                content={children.map((child) => renderChild(child))}
            />
        );
    },
    Region: ({ props, children, renderChild }) => {
        const region = as<RegionProps>(props);
        return (
            <RegionView
                title={region.title}
                layout={region.layout}
                columns={region.columns}
                tone={region.tone}
                variant={region.variant}
                content={children.map((child) => renderChild(child))}
            />
        );
    },
    Scene3D: (renderer) => <Scene3D {...as<Scene3DProps>(renderer.props)} fallback={homeView(renderer)} />,
};

/** Renderers of the insight catalog, for `DrawnView`'s `renderers`. */
export const insightRenderers: Renderers = { [INSIGHT_CATALOG_ID]: insight };

/** Renderers of the studio catalog, for `DrawnView`'s `renderers`. */
export const studioRenderers: Renderers = { [STUDIO_CATALOG_ID]: studio };

/**
 * Renderers of every adk-ui catalog, for `DrawnView`'s `renderers`:
 *
 * ```tsx
 * <DrawnView surface={surface} onSend={send} renderers={adkUiRenderers} />
 * ```
 */
export const adkUiRenderers: Renderers = { ...insightRenderers, ...studioRenderers };
