import React, { startTransition, useEffect, useState } from 'react';
import { Renderer } from './Renderer';
import type { Component, UiEvent } from './types';
import {
    AdkUiKitProvider,
    defaultUiKit,
    resolveApprovedUiKit,
    type UiKitManifest,
    type UiKitSlots,
} from './kit';
import { sanitizeUrl } from './security';

export type UiPageTemplate = 'marketing' | 'dashboard' | 'workspace' | 'detail' | 'form' | 'immersive';
export type UiPageAtmosphere = 'clean' | 'grid' | 'glow' | 'gradient' | 'spatial';
export type UiRegionLayout = 'stack' | 'grid' | 'bento' | 'split' | 'editorial' | 'stats';
export type UiRegionTone = 'default' | 'muted' | 'accent' | 'contrast';

export interface UiNavigationItem { label: string; route: string; icon?: string }
export interface UiPageHero {
    title: string;
    description?: string;
    content?: Component[];
    actions?: Component[];
    visual?: Component[];
    alignment?: 'split' | 'center' | 'editorial';
}
export interface UiPageRegion {
    id: string;
    title?: string;
    description?: string;
    layout?: UiRegionLayout;
    columns?: number;
    tone?: UiRegionTone;
    components: Component[];
}
export interface UiApplicationPage {
    id: string;
    route: string;
    title: string;
    description?: string;
    template?: UiPageTemplate;
    eyebrow?: string;
    hero?: UiPageHero;
    regions: UiPageRegion[];
    aside?: Component[];
    footer?: Component[];
    atmosphere?: UiPageAtmosphere;
}
export interface UiApplication {
    id: string;
    name: string;
    version?: number;
    owner?: string;
    updated_at?: string;
    kit_id?: string;
    initial_route: string;
    navigation?: UiNavigationItem[];
    pages: UiApplicationPage[];
    state?: Record<string, unknown>;
}
export type UiApplicationUpdate =
    | { operation: 'navigate'; route: string }
    | { operation: 'replace'; application: UiApplication }
    | { operation: 'upsert_page'; page: UiApplicationPage }
    | { operation: 'remove_page'; page_id: string }
    | { operation: 'patch_state'; values: Record<string, unknown> };

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null && !Array.isArray(value);
}

export function validateUiApplication(application: UiApplication): string[] {
    const errors: string[] = [];
    if (!application.id?.trim()) errors.push('id must not be empty');
    if (!application.name?.trim()) errors.push('name must not be empty');
    if (!Array.isArray(application.pages) || application.pages.length === 0) errors.push('pages must contain at least one page');
    if (application.pages.length > 24) errors.push('pages must not exceed 24');
    const ids = new Set<string>();
    const routes = new Set<string>();
    for (const [index, page] of application.pages.entries()) {
        if (!page.id?.trim() || ids.has(page.id)) errors.push(`pages[${index}].id must be non-empty and unique`);
        ids.add(page.id);
        if (!page.route?.startsWith('/') || page.route.startsWith('//') || page.route.includes('..') || routes.has(page.route)) {
            errors.push(`pages[${index}].route must be a unique absolute application route`);
        }
        routes.add(page.route);
        if (!Array.isArray(page.regions) || page.regions.length > 16) errors.push(`pages[${index}].regions must be an array with at most 16 entries`);
        for (const [regionIndex, region] of page.regions.entries()) {
            const columns = region.columns ?? 3;
            if (columns < 1 || columns > 6) errors.push(`pages[${index}].regions[${regionIndex}].columns must be between 1 and 6`);
            if (!Array.isArray(region.components)) errors.push(`pages[${index}].regions[${regionIndex}].components must be an array`);
        }
    }
    if (!routes.has(application.initial_route)) errors.push('initial_route must reference an application page');
    for (const [index, item] of (application.navigation ?? []).entries()) {
        if (!item.label?.trim()) errors.push(`navigation[${index}].label must not be empty`);
        if (!routes.has(item.route)) errors.push(`navigation[${index}].route must reference an application page`);
    }
    return errors;
}

export function parseUiApplication(value: unknown): UiApplication | null {
    const candidate = isRecord(value) && isRecord(value.application) ? value.application : value;
    if (!isRecord(candidate) || typeof candidate.id !== 'string' || typeof candidate.name !== 'string'
        || typeof candidate.initial_route !== 'string' || !Array.isArray(candidate.pages)) return null;
    const application = candidate as unknown as UiApplication;
    return validateUiApplication(application).length === 0 ? application : null;
}

export function applyUiApplicationUpdate(application: UiApplication, update: UiApplicationUpdate): UiApplication {
    if (update.operation === 'replace') return {
        ...update.application,
        version: (application.version ?? 1) + 1,
        owner: update.application.owner || application.owner,
        updated_at: new Date().toISOString(),
    };
    if (update.operation === 'patch_state') {
        return touchApplication({ ...application, state: { ...application.state, ...update.values } });
    }
    if (update.operation === 'upsert_page') {
        const exists = application.pages.some((page) => page.id === update.page.id);
        const previousRoute = application.pages.find((page) => page.id === update.page.id)?.route;
        return touchApplication({
            ...application,
            pages: exists
                ? application.pages.map((page) => page.id === update.page.id ? update.page : page)
                : [...application.pages, update.page],
            initial_route: previousRoute && application.initial_route === previousRoute ? update.page.route : application.initial_route,
            navigation: application.navigation?.map((item) => previousRoute && item.route === previousRoute ? { ...item, route: update.page.route } : item),
        });
    }
    if (update.operation === 'remove_page') {
        const pages = application.pages.filter((page) => page.id !== update.page_id);
        const routes = new Set(pages.map((page) => page.route));
        return touchApplication({ ...application, pages, navigation: application.navigation?.filter((item) => routes.has(item.route)) });
    }
    return application;
}

function touchApplication(application: UiApplication): UiApplication {
    return {
        ...application,
        version: (application.version ?? 1) + 1,
        updated_at: new Date().toISOString(),
    };
}

export interface ApplicationRendererProps {
    application: UiApplication;
    updates?: UiApplicationUpdate | UiApplicationUpdate[];
    kits?: readonly UiKitManifest[];
    fallbackKit?: UiKitManifest;
    slots?: UiKitSlots;
    onAction?: (event: UiEvent) => void;
    onNavigate?: (route: string, page: UiApplicationPage) => void;
}

function ComponentSet({ components, onAction }: { components: Component[]; onAction?: (event: UiEvent) => void }) {
    return <>{components.map((component, index) => <Renderer key={component.id ?? `${component.type}-${index}`} component={component} onAction={onAction} />)}</>;
}

export function ApplicationRenderer({
    application,
    updates,
    kits = [],
    fallbackKit = defaultUiKit,
    slots,
    onAction,
    onNavigate,
}: ApplicationRendererProps) {
    const [runtime, setRuntime] = useState(application);
    const [activeRoute, setActiveRoute] = useState(application.initial_route);
    const [visitedRoutes, setVisitedRoutes] = useState(() => new Set([application.initial_route]));

    useEffect(() => {
        setRuntime(application);
        setActiveRoute(application.initial_route);
        setVisitedRoutes(new Set([application.initial_route]));
    }, [application]);

    useEffect(() => {
        const list = updates ? (Array.isArray(updates) ? updates : [updates]) : [];
        if (list.length === 0) return;
        setRuntime((current) => {
            const candidate = list.reduce(applyUiApplicationUpdate, current);
            return validateUiApplication(candidate).length === 0 ? candidate : current;
        });
        const navigation = [...list].reverse().find((update) => update.operation === 'navigate');
        const replacement = [...list].reverse().find((update) => update.operation === 'replace');
        const nextRoute = navigation?.operation === 'navigate'
            ? navigation.route
            : replacement?.operation === 'replace'
                ? replacement.application.initial_route
                : null;
        if (nextRoute) {
            setActiveRoute(nextRoute);
            setVisitedRoutes((current) => new Set(current).add(nextRoute));
        }
    }, [updates]);

    const kit = resolveApprovedUiKit(runtime.kit_id, kits, fallbackKit);
    const errors = validateUiApplication(runtime);
    const activePage = runtime.pages.find((page) => page.route === activeRoute) ?? runtime.pages[0];

    const navigate = (route: string) => {
        const page = runtime.pages.find((candidate) => candidate.route === route);
        if (!page) return;
        startTransition(() => {
            setActiveRoute(route);
            setVisitedRoutes((current) => new Set(current).add(route));
        });
        onNavigate?.(route, page);
    };

    const handleAction = (event: UiEvent) => {
        if (
            (event.action === 'button_click' || event.action === 'form_submit')
            && event.action_id.startsWith('navigate:')
        ) {
            if (event.action === 'form_submit') {
                onAction?.(event);
            }
            navigate(event.action_id.slice('navigate:'.length));
            return;
        }
        onAction?.(event);
    };

    if (errors.length > 0 || !activePage) {
        return <div className="adk-app-error" role="alert">Cannot render application: {errors[0] ?? 'no active page'}</div>;
    }

    const logo = kit.assets.find((asset) => asset.kind === 'logo');
    const safeLogo = logo ? sanitizeUrl(logo.uri, 'image') : null;
    const navigation: UiNavigationItem[] = runtime.navigation ?? runtime.pages.map((page) => ({ label: page.title, route: page.route }));

    return (
        <AdkUiKitProvider kit={kit} slots={slots}>
            <style data-adk-ui-application-styles>{ADK_UI_APPLICATION_CSS}</style>
            <div className="adk-app" data-application={runtime.id}>
                <header className="adk-app__header">
                    <button className="adk-app__brand" type="button" onClick={() => navigate(runtime.initial_route)} aria-label={`${runtime.name} home`}>
                        {safeLogo ? <img src={safeLogo} alt={logo?.alt ?? ''} /> : <span>{runtime.name.slice(0, 1).toUpperCase()}</span>}
                        <strong>{runtime.name}</strong>
                    </button>
                    <nav className="adk-app__nav" aria-label="Application navigation">
                        {navigation.map((item) => (
                            <button key={`${item.route}-${item.label}`} type="button" onClick={() => navigate(item.route)} aria-current={activeRoute === item.route ? 'page' : undefined}>
                                {item.icon && <span aria-hidden="true">{item.icon}</span>}{item.label}
                            </button>
                        ))}
                    </nav>
                    <span className="adk-app__live"><i />Live agent</span>
                </header>
                <div className="adk-app__pages">
                    {runtime.pages.filter((page) => visitedRoutes.has(page.route)).map((page) => (
                        <main
                            key={page.id}
                            className={`adk-app-page adk-app-page--${page.template ?? 'dashboard'} adk-app-atmosphere--${page.atmosphere ?? 'clean'}`}
                            hidden={page.route !== activeRoute}
                            aria-label={page.title}
                        >
                            <div className="adk-app-page__orb" aria-hidden="true" />
                            {page.hero ? (
                                <section className={`adk-app-hero adk-app-hero--${page.hero.alignment ?? 'split'}`}>
                                    <div className="adk-app-hero__copy">
                                        {page.eyebrow && <span className="adk-app-eyebrow">{page.eyebrow}</span>}
                                        <h1>{page.hero.title}</h1>
                                        {page.hero.description && <p>{page.hero.description}</p>}
                                        <ComponentSet components={page.hero.content ?? []} onAction={handleAction} />
                                        {page.hero.actions && <div className="adk-app-hero__actions"><ComponentSet components={page.hero.actions} onAction={handleAction} /></div>}
                                    </div>
                                    {page.hero.visual && page.hero.visual.length > 0 && (
                                        <div className="adk-app-hero__visual"><ComponentSet components={page.hero.visual} onAction={handleAction} /></div>
                                    )}
                                </section>
                            ) : (
                                <header className="adk-app-page__intro">
                                    {page.eyebrow && <span className="adk-app-eyebrow">{page.eyebrow}</span>}
                                    <h1>{page.title}</h1>
                                    {page.description && <p>{page.description}</p>}
                                </header>
                            )}
                            <div className={page.aside?.length ? 'adk-app-page__with-aside' : undefined}>
                                <div className="adk-app-page__regions">
                                    {page.regions.map((region) => (
                                        <section key={region.id} className={`adk-app-region adk-app-region--${region.layout ?? 'stack'} adk-app-region-tone--${region.tone ?? 'default'}`}>
                                            {(region.title || region.description) && <header><div>{region.title && <h2>{region.title}</h2>}{region.description && <p>{region.description}</p>}</div><span>{String(page.regions.indexOf(region) + 1).padStart(2, '0')}</span></header>}
                                            <div className="adk-app-region__content" style={{ '--adk-app-columns': Math.min(6, Math.max(1, region.columns ?? 3)) } as React.CSSProperties}>
                                                <ComponentSet components={region.components} onAction={handleAction} />
                                            </div>
                                        </section>
                                    ))}
                                </div>
                                {page.aside && page.aside.length > 0 && <aside className="adk-app-page__aside"><ComponentSet components={page.aside} onAction={handleAction} /></aside>}
                            </div>
                            {page.footer && page.footer.length > 0 && <footer className="adk-app-page__footer"><ComponentSet components={page.footer} onAction={handleAction} /></footer>}
                        </main>
                    ))}
                </div>
            </div>
        </AdkUiKitProvider>
    );
}

export const ADK_UI_APPLICATION_CSS = `
.adk-app { position: relative; min-height: 720px; overflow: hidden; container-type: inline-size; color: var(--adk-ui-foreground); background: var(--adk-ui-background); font-family: var(--adk-ui-font-body); }
.adk-app * { box-sizing: border-box; }
.adk-app__header { position: sticky; z-index: 20; top: 0; display: grid; grid-template-columns: minmax(180px, 1fr) auto minmax(180px, 1fr); align-items: center; min-height: 72px; padding: 0 3.5vw; border-bottom: 1px solid color-mix(in srgb, var(--adk-ui-border) 76%, transparent); background: color-mix(in srgb, var(--adk-ui-background) 88%, transparent); backdrop-filter: blur(20px); }
.adk-app__brand { display: flex; align-items: center; width: max-content; gap: 10px; border: 0; background: transparent; color: inherit; cursor: pointer; font-family: var(--adk-ui-font-heading); }
.adk-app__brand img, .adk-app__brand > span { display: grid; width: 32px; height: 32px; place-items: center; border-radius: 10px; background: var(--adk-ui-primary); color: var(--adk-ui-primary-foreground); object-fit: contain; }
.adk-app__nav { display: flex; align-items: center; gap: 5px; padding: 5px; border: 1px solid var(--adk-ui-border); border-radius: 999px; background: color-mix(in srgb, var(--adk-ui-surface) 82%, transparent); box-shadow: 0 8px 30px color-mix(in srgb, var(--adk-ui-foreground) 5%, transparent); }
.adk-app__nav button { display: flex; align-items: center; gap: 6px; border: 0; border-radius: 999px; padding: 8px 13px; background: transparent; color: var(--adk-ui-muted-foreground); cursor: pointer; font: 600 0.78rem var(--adk-ui-font-body); }
.adk-app__nav button[aria-current=page] { background: var(--adk-ui-foreground); color: var(--adk-ui-background); }
.adk-app__live { justify-self: end; display: flex; align-items: center; gap: 7px; color: var(--adk-ui-muted-foreground); font-size: 0.74rem; font-weight: 650; }
.adk-app__live i { width: 7px; height: 7px; border-radius: 50%; background: var(--adk-ui-success); box-shadow: 0 0 0 5px color-mix(in srgb, var(--adk-ui-success) 14%, transparent); }
.adk-app-page { position: relative; min-height: 648px; padding: clamp(48px, 7vw, 104px) clamp(22px, 6vw, 96px); isolation: isolate; }
.adk-app-page[hidden] { display: none; }
.adk-app-page__orb { position: absolute; z-index: -1; top: -22%; right: -8%; width: min(54vw, 780px); aspect-ratio: 1; border-radius: 50%; background: radial-gradient(circle, color-mix(in srgb, var(--adk-ui-primary) 20%, transparent), transparent 66%); filter: blur(18px); pointer-events: none; }
.adk-app-atmosphere--grid { background-image: linear-gradient(color-mix(in srgb, var(--adk-ui-border) 36%, transparent) 1px, transparent 1px), linear-gradient(90deg, color-mix(in srgb, var(--adk-ui-border) 36%, transparent) 1px, transparent 1px); background-size: 48px 48px; }
.adk-app-atmosphere--gradient { background: linear-gradient(145deg, var(--adk-ui-background), color-mix(in srgb, var(--adk-ui-secondary) 52%, var(--adk-ui-background))); }
.adk-app-atmosphere--spatial { background: radial-gradient(circle at 70% 12%, color-mix(in srgb, var(--adk-ui-primary) 18%, transparent), transparent 34%), linear-gradient(155deg, var(--adk-ui-background), var(--adk-ui-muted)); }
.adk-app-hero { display: grid; grid-template-columns: minmax(0, .88fr) minmax(380px, 1.12fr); align-items: center; max-width: 1440px; min-height: 500px; margin: 0 auto clamp(64px, 9vw, 128px); gap: clamp(40px, 7vw, 112px); }
.adk-app-hero--center { grid-template-columns: minmax(0, 900px); justify-content: center; text-align: center; }
.adk-app-hero--center .adk-app-hero__actions { justify-content: center; }
.adk-app-hero--editorial { grid-template-columns: minmax(0, 1.3fr) minmax(300px, .7fr); align-items: end; }
.adk-app-eyebrow { display: inline-flex; margin-bottom: 18px; color: var(--adk-ui-primary); font: 750 .72rem/1 var(--adk-ui-font-mono); letter-spacing: .16em; text-transform: uppercase; }
.adk-app-hero h1, .adk-app-page__intro h1 { max-width: 920px; margin: 0; color: var(--adk-ui-foreground); font: 700 clamp(2.8rem, 8cqw, 6.6rem)/.94 var(--adk-ui-font-heading); letter-spacing: -.065em; text-wrap: balance; }
.adk-app-hero__copy > p, .adk-app-page__intro > p { max-width: 680px; margin: 26px 0 0; color: var(--adk-ui-muted-foreground); font-size: clamp(1rem, 1.35vw, 1.28rem); line-height: 1.65; }
.adk-app-hero__actions { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 30px; }
.adk-app-hero__visual { min-width: 0; transform: perspective(1400px) rotateY(-2deg) rotateX(1deg); }
.adk-app-page__intro { max-width: 1440px; margin: 0 auto clamp(50px, 7vw, 90px); }
.adk-app-page__intro h1 { font-size: clamp(2.8rem, 7.2cqw, 5.6rem); }
.adk-app-page__with-aside { display: grid; grid-template-columns: minmax(0, 1fr) minmax(250px, 330px); align-items: start; max-width: 1440px; margin: auto; gap: clamp(28px, 5vw, 72px); }
.adk-app-page__regions { display: grid; max-width: 1440px; margin: auto; gap: clamp(48px, 7vw, 96px); }
.adk-app-region { min-width: 0; }
.adk-app-region > header { display: flex; align-items: start; justify-content: space-between; margin-bottom: 24px; padding-bottom: 16px; border-bottom: 1px solid var(--adk-ui-border); }
.adk-app-region > header h2 { margin: 0; color: var(--adk-ui-foreground); font: 680 clamp(1.55rem, 2.5vw, 2.6rem)/1.08 var(--adk-ui-font-heading); letter-spacing: -.035em; }
.adk-app-region > header p { max-width: 650px; margin: 8px 0 0; color: var(--adk-ui-muted-foreground); line-height: 1.55; }
.adk-app-region > header > span { color: var(--adk-ui-primary); font: 700 .72rem var(--adk-ui-font-mono); }
.adk-app-region__content { display: grid; gap: clamp(14px, 2vw, 24px); }
.adk-app-region--grid .adk-app-region__content, .adk-app-region--stats .adk-app-region__content { grid-template-columns: repeat(var(--adk-app-columns), minmax(0, 1fr)); }
.adk-app-region--bento .adk-app-region__content { grid-template-columns: repeat(12, 1fr); grid-auto-flow: dense; }
.adk-app-region--bento .adk-app-region__content > * { grid-column: span 4; min-width: 0; }
.adk-app-region--bento .adk-app-region__content > :first-child { grid-column: span 8; grid-row: span 2; }
.adk-app-region--bento .adk-app-region__content > :nth-child(4n) { grid-column: span 6; }
.adk-app-region--split .adk-app-region__content { grid-template-columns: repeat(2, minmax(0, 1fr)); align-items: center; }
.adk-app-region--editorial .adk-app-region__content { grid-template-columns: minmax(260px, .7fr) minmax(0, 1.3fr); align-items: start; }
.adk-app-region-tone--muted, .adk-app-region-tone--accent, .adk-app-region-tone--contrast { margin-inline: clamp(-28px, -3vw, -60px); padding: clamp(28px, 4vw, 60px); border-radius: calc(var(--adk-ui-radius) * 2); }
.adk-app-region-tone--muted { background: var(--adk-ui-muted); }
.adk-app-region-tone--accent { background: color-mix(in srgb, var(--adk-ui-secondary) 72%, var(--adk-ui-surface)); }
.adk-app-region-tone--contrast { background: var(--adk-ui-foreground); color: var(--adk-ui-background); }
.adk-app-region-tone--contrast h2, .adk-app-region-tone--contrast .adk-ui-kit-surface { color: var(--adk-ui-background); }
.adk-app-page__aside { position: sticky; top: 100px; padding: 20px; border: 1px solid var(--adk-ui-border); border-radius: calc(var(--adk-ui-radius) * 1.5); background: color-mix(in srgb, var(--adk-ui-surface) 88%, transparent); backdrop-filter: blur(16px); }
.adk-app-page__footer { max-width: 1440px; margin: 72px auto 0; padding-top: 28px; border-top: 1px solid var(--adk-ui-border); }
.adk-app-page--immersive { padding-inline: clamp(14px, 3vw, 48px); }
.adk-app-page--immersive .adk-app-hero { grid-template-columns: minmax(270px, .55fr) minmax(0, 1.45fr); max-width: 1600px; }
.adk-app-error { padding: 24px; border: 1px solid #f5b8b1; border-radius: 12px; background: #fff0ee; color: #8f1d14; }
@media (max-width: 900px) {
  .adk-app__header { grid-template-columns: 1fr auto; padding: 12px 18px; }
  .adk-app__nav { grid-column: 1 / -1; width: 100%; margin-top: 10px; overflow-x: auto; order: 3; }
  .adk-app__live { display: none; }
  .adk-app-hero, .adk-app-page--immersive .adk-app-hero { grid-template-columns: 1fr; min-height: 0; }
  .adk-app-hero h1, .adk-app-page__intro h1 { font-size: clamp(2.8rem, 14vw, 5.4rem); }
  .adk-app-page__with-aside, .adk-app-region--split .adk-app-region__content, .adk-app-region--editorial .adk-app-region__content { grid-template-columns: 1fr; }
  .adk-app-region--grid .adk-app-region__content, .adk-app-region--stats .adk-app-region__content { grid-template-columns: repeat(min(2, var(--adk-app-columns)), minmax(0, 1fr)); }
  .adk-app-region--bento .adk-app-region__content > *, .adk-app-region--bento .adk-app-region__content > :first-child, .adk-app-region--bento .adk-app-region__content > :nth-child(4n) { grid-column: span 12; }
  .adk-app-page__aside { position: static; }
}
@container (max-width: 900px) {
  .adk-app__header { grid-template-columns: 1fr; align-items: start; gap: 12px; padding: 16px 20px; }
  .adk-app__nav { width: 100%; overflow-x: auto; }
  .adk-app__live { display: none; }
  .adk-app-page { padding: 54px clamp(18px, 6cqw, 42px); }
  .adk-app-hero, .adk-app-page--immersive .adk-app-hero { grid-template-columns: 1fr; min-height: 0; gap: 36px; }
  .adk-app-hero h1, .adk-app-page__intro h1 { font-size: clamp(2.7rem, 11cqw, 5.4rem); }
  .adk-app-page__with-aside, .adk-app-region--split .adk-app-region__content, .adk-app-region--editorial .adk-app-region__content { grid-template-columns: 1fr; }
  .adk-app-region--grid .adk-app-region__content, .adk-app-region--stats .adk-app-region__content { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .adk-app-region--bento .adk-app-region__content > *, .adk-app-region--bento .adk-app-region__content > :first-child, .adk-app-region--bento .adk-app-region__content > :nth-child(4n) { grid-column: span 12; }
  .adk-app-page__aside { position: static; }
}
@container (max-width: 520px) {
  .adk-app__brand strong { max-width: 140px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .adk-app-region--grid .adk-app-region__content, .adk-app-region--stats .adk-app-region__content { grid-template-columns: 1fr; }
  .adk-app-region-tone--muted, .adk-app-region-tone--accent, .adk-app-region-tone--contrast { margin-inline: -8px; padding: 22px 14px; }
}
@media (max-width: 560px) {
  .adk-app-page { padding-inline: 18px; }
  .adk-app__brand strong { max-width: 140px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .adk-app-region--grid .adk-app-region__content, .adk-app-region--stats .adk-app-region__content { grid-template-columns: 1fr; }
  .adk-app-region-tone--muted, .adk-app-region-tone--accent, .adk-app-region-tone--contrast { margin-inline: -8px; padding: 22px 14px; }
}
`;
