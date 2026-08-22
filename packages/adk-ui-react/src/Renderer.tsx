import React, { createContext, lazy, useContext, useMemo, useRef, useState } from 'react';
import type { Component, DataSource, UiEvent, UiUpdate } from './types';
import { applyUiUpdates } from './updates';
import { AlertCircle, CheckCircle, Info, XCircle, User, Mail, Calendar } from 'lucide-react';
import Markdown, { type Components } from 'react-markdown';
import clsx from 'clsx';
import { isExternalNavigationUrl, sanitizeUrl } from './security';
import {
    ADK_UI_KIT_CSS,
    kitStyleVariables,
    resolveKitAssetSource,
    useAdkUiKit,
} from './kit';
import { OptionalRenderer } from './OptionalRenderer';

const ChartRenderer = lazy(() => import('./ChartRenderer'));
const Scene3DRenderer = lazy(() => import('./Scene3DRenderer'));

const IconMap: Record<string, React.ComponentType<any>> = {
    'alert-circle': AlertCircle,
    'check-circle': CheckCircle,
    'info': Info,
    'x-circle': XCircle,
    'user': User,
    'mail': Mail,
    'calendar': Calendar,
};

function KitMarkdownImage({ node: _node, src, alt, ...props }: React.ComponentProps<'img'> & { node?: unknown }) {
    const { manifest } = useAdkUiKit();
    const resolvedSrc = src ? resolveKitAssetSource(manifest, src) : null;
    const safeSrc = sanitizeUrl(resolvedSrc, 'image');
    if (!safeSrc) {
        return null;
    }

    return (
        <img
            {...props}
            alt={alt ?? ''}
            className="max-w-full h-auto rounded-lg"
            loading="lazy"
            referrerPolicy="no-referrer"
            src={safeSrc}
        />
    );
}

const markdownComponents: Components = {
    a({ node: _node, href, children, ...props }) {
        const safeHref = sanitizeUrl(href, 'anchor');
        if (!safeHref) {
            return <span>{children}</span>;
        }

        const isExternal = isExternalNavigationUrl(safeHref);
        return (
            <a
                {...props}
                href={safeHref}
                rel={isExternal ? 'noopener noreferrer nofollow' : undefined}
                target={isExternal ? '_blank' : undefined}
            >
                {children}
            </a>
        );
    },
    img: KitMarkdownImage,
};

function renderBlockedAsset(kind: 'image' | 'video' | 'audio') {
    return <div className="text-sm text-red-600 dark:text-red-400">Blocked unsafe {kind} URL.</div>;
}

function humanizeActionId(actionId: string): string {
    const normalized = actionId.replace(/[_-]+/g, ' ').replace(/\s+/g, ' ').trim();
    return normalized ? normalized.replace(/\b\w/g, (char) => char.toUpperCase()) : 'Run Action';
}

function resolveGap(gap: number | string | undefined): string {
    if (typeof gap === 'number') {
        return `${Math.max(0, gap <= 12 ? gap * 4 : gap)}px`;
    }
    return typeof gap === 'string' && gap.trim() ? gap : '16px';
}

function mapFlexPosition(value: string | undefined): React.CSSProperties['justifyContent'] {
    const normalized = value?.trim().toLowerCase();
    if (normalized === 'start' || normalized === 'flex-start') return 'flex-start';
    if (normalized === 'end' || normalized === 'flex-end') return 'flex-end';
    if (normalized === 'center') return 'center';
    if (normalized === 'between' || normalized === 'space-between') return 'space-between';
    if (normalized === 'around' || normalized === 'space-around') return 'space-around';
    if (normalized === 'evenly' || normalized === 'space-evenly') return 'space-evenly';
    return undefined;
}

function mapFlexAlign(value: string | undefined): React.CSSProperties['alignItems'] {
    const normalized = value?.trim().toLowerCase();
    if (normalized === 'start' || normalized === 'flex-start') return 'flex-start';
    if (normalized === 'end' || normalized === 'flex-end') return 'flex-end';
    if (normalized === 'center') return 'center';
    if (normalized === 'stretch') return 'stretch';
    if (normalized === 'baseline') return 'baseline';
    return undefined;
}

// Context for form handling
interface FormContextValue {
    onAction?: (event: UiEvent) => void;
    formRef?: React.RefObject<HTMLFormElement | null>;
}

const FormContext = createContext<FormContextValue>({});

interface DataSourceContextValue {
    dataModel?: unknown;
    onRefresh?: (source: DataSource) => void;
}

const DataSourceContext = createContext<DataSourceContextValue>({});

export interface DataSourceProviderProps extends DataSourceContextValue {
    children: React.ReactNode;
}

export function DataSourceProvider({ dataModel, onRefresh, children }: DataSourceProviderProps) {
    const value = useMemo(() => ({ dataModel, onRefresh }), [dataModel, onRefresh]);
    return <DataSourceContext.Provider value={value}>{children}</DataSourceContext.Provider>;
}

function resolveJsonPointer(value: unknown, pointer: string): unknown {
    if (pointer === '' || pointer === '/') return value;
    if (!pointer.startsWith('/')) return undefined;
    return pointer.slice(1).split('/').reduce<unknown>((current, segment) => {
        if (current === null || typeof current !== 'object') return undefined;
        const key = segment.replace(/~1/g, '/').replace(/~0/g, '~');
        return (current as Record<string, unknown>)[key];
    }, value);
}

interface RendererProps {
    component: Component;
    onAction?: (event: UiEvent) => void;
    /** Theme for this component: 'dark' wraps in dark mode styling */
    theme?: 'light' | 'dark' | 'system';
    dataModel?: unknown;
    onDataRefresh?: (source: DataSource) => void;
}

export const Renderer: React.FC<RendererProps> = ({ component, onAction, theme, dataModel, onDataRefresh }) => {
    const isDark = theme === 'dark';
    const contextValue = useMemo(() => ({ onAction }), [onAction]);
    const kit = useAdkUiKit();

    return (
        <FormContext.Provider value={contextValue}>
          <DataSourceProvider dataModel={dataModel} onRefresh={onDataRefresh}>
            {!kit.provided && <style data-adk-ui-kit-styles>{ADK_UI_KIT_CSS}</style>}
            <div
                className={clsx('adk-ui-kit-surface', isDark && 'dark')}
                data-adk-ui-kit={kit.manifest.id}
                style={kitStyleVariables(kit.manifest)}
            >
                <ComponentRenderer component={component} />
            </div>
          </DataSourceProvider>
        </FormContext.Provider>
    );
};

interface StreamingRendererProps extends RendererProps {
    updates?: UiUpdate | UiUpdate[];
}

export const StreamingRenderer: React.FC<StreamingRendererProps> = ({ component, updates, onAction, theme, dataModel, onDataRefresh }) => {
    const [current, setCurrent] = useState(component);
    const updatesList = useMemo(() => {
        if (!updates) return [];
        return Array.isArray(updates) ? updates : [updates];
    }, [updates]);

    React.useEffect(() => {
        setCurrent(component);
    }, [component]);

    React.useEffect(() => {
        if (updatesList.length === 0) return;
        setCurrent((prev) => {
            const updated = applyUiUpdates(prev, updatesList);
            return updated ?? prev;
        });
    }, [updatesList]);

    return <Renderer component={current} onAction={onAction} theme={theme} dataModel={dataModel} onDataRefresh={onDataRefresh} />;
};

function componentKey(component: Component, index: number): string {
    return component.id ?? `${component.type}-${index}`;
}

function formControlId(component: { id?: string; name: string; type: string }): string {
    return `adk-ui-${component.id ?? `${component.type}-${component.name}`}`.replace(/[^a-zA-Z0-9_-]/g, '-');
}

function containsInput(component: Component): boolean {
    if (['text_input', 'number_input', 'select', 'multi_select', 'switch', 'date_input', 'slider', 'textarea'].includes(component.type)) {
        return true;
    }
    if (component.type === 'stack' || component.type === 'grid' || component.type === 'container') {
        return component.children.some(containsInput);
    }
    if (component.type === 'card' || component.type === 'modal') {
        return component.content.some(containsInput) || (component.footer?.some(containsInput) ?? false);
    }
    if (component.type === 'tabs') {
        return component.tabs.some((tab) => tab.content.some(containsInput));
    }
    return false;
}

function readFormData(form: HTMLFormElement): Record<string, unknown> {
    const formData = new FormData(form);
    const data: Record<string, unknown> = {};
    for (const key of new Set(formData.keys())) {
        const values = formData.getAll(key);
        data[key] = values.length > 1 ? values : values[0];
    }
    return data;
}

const ComponentRenderer: React.FC<{ component: Component }> = React.memo(({ component }) => {
    const { slots } = useAdkUiKit();
    const Slot = slots[component.type];
    if (Slot) {
        return <Slot component={component} renderDefault={() => <DefaultComponentRenderer component={component} />} />;
    }
    return <DefaultComponentRenderer component={component} />;
});

const DefaultComponentRenderer: React.FC<{ component: Component }> = React.memo(({ component }) => {
    const { onAction, formRef } = useContext(FormContext);
    const { manifest } = useAdkUiKit();
    const cardFormRef = useRef<HTMLFormElement>(null);
    const [activeTab, setActiveTab] = React.useState(0);
    const [sortColumn, setSortColumn] = React.useState<string | null>(null);
    const [sortDirection, setSortDirection] = React.useState<'asc' | 'desc'>('asc');
    const [currentPage, setCurrentPage] = React.useState(0);
    const dataContext = useContext(DataSourceContext);
    const onDataRefresh = dataContext.onRefresh;
    const dataSource = 'data_source' in component ? component.data_source : undefined;
    const boundData = useMemo(
        () => dataSource ? resolveJsonPointer(dataContext.dataModel, dataSource.binding_path) : undefined,
        [dataContext.dataModel, dataSource],
    );

    React.useEffect(() => {
        if (!dataSource || !onDataRefresh) return;
        const interval = dataSource.refresh?.mode === 'interval' ? dataSource.refresh.secs : undefined;
        const seconds = interval ?? dataSource.ttl;
        if (!seconds || seconds <= 0) return;
        const timer = window.setInterval(() => onDataRefresh(dataSource), seconds * 1000);
        return () => window.clearInterval(timer);
    }, [onDataRefresh, dataSource]);

    const handleButtonClick = (actionId: string, variant?: string) => {
        const submitsForm = Boolean(formRef?.current)
            && (variant === undefined || variant === 'primary' || variant === 'danger');

        if (submitsForm && formRef?.current) {
            if (!formRef.current.reportValidity()) {
                return;
            }
            onAction?.({
                action: 'form_submit',
                action_id: actionId,
                data: readFormData(formRef.current),
                source_component_id: component.id ?? actionId,
            });
        } else {
            onAction?.({
                action: 'button_click',
                action_id: actionId,
                source_component_id: component.id ?? actionId,
            });
        }
    };

    switch (component.type) {
        case 'text':
            // Use Markdown for body text to render formatted content
            if (component.variant === 'body' || !component.variant) {
                return (
                    <div className="adk-ui-body prose prose-sm dark:prose-invert max-w-none text-gray-700 dark:text-gray-300">
                        <Markdown components={markdownComponents} skipHtml>{component.content}</Markdown>
                    </div>
                );
            }
            const Tag = component.variant === 'h1' ? 'h1' :
                component.variant === 'h2' ? 'h2' :
                    component.variant === 'h3' ? 'h3' :
                        component.variant === 'h4' ? 'h4' :
                            component.variant === 'code' ? 'code' : 'p';
            const classes = clsx({
                'text-4xl font-bold tracking-tight mb-4 text-slate-950 dark:text-white': component.variant === 'h1',
                'text-3xl font-bold tracking-tight mb-3 text-slate-950 dark:text-white': component.variant === 'h2',
                'text-2xl font-bold tracking-tight mb-2 text-slate-900 dark:text-white': component.variant === 'h3',
                'text-xl font-semibold tracking-tight mb-2 text-slate-900 dark:text-white': component.variant === 'h4',
                'font-mono bg-gray-100 dark:bg-gray-800 p-1 rounded dark:text-gray-100': component.variant === 'code',
                'text-sm text-gray-500 dark:text-gray-400': component.variant === 'caption',
            });
            return <Tag className={clsx(classes, component.variant === 'caption' ? 'adk-ui-caption' : 'adk-ui-heading')}>{component.content}</Tag>;

        case 'button':
            const buttonLabel = component.label?.trim() || humanizeActionId(component.action_id);
            const btnClasses = clsx('adk-ui-button inline-flex items-center justify-center px-4 py-2.5 rounded-lg font-semibold transition-colors border border-transparent', {
                'adk-ui-button--primary': component.variant === 'primary' || !component.variant,
                'adk-ui-button--secondary': component.variant === 'secondary',
                'adk-ui-button--danger': component.variant === 'danger',
                'adk-ui-button--ghost': component.variant === 'ghost',
                'adk-ui-button--outline': component.variant === 'outline',
                'bg-teal-700 text-white hover:bg-teal-800 shadow-sm': component.variant === 'primary' || !component.variant,
                'bg-gray-200 text-gray-800 hover:bg-gray-300': component.variant === 'secondary',
                'bg-red-600 text-white hover:bg-red-700': component.variant === 'danger',
                'bg-transparent hover:bg-gray-100': component.variant === 'ghost',
                'border border-gray-300 hover:bg-gray-50': component.variant === 'outline',
                'opacity-50 cursor-not-allowed': component.disabled,
            });
            return (
                <button
                    type="button"
                    className={btnClasses}
                    disabled={component.disabled}
                    onClick={() => handleButtonClick(component.action_id, component.variant)}
                >
                    {buttonLabel}
                </button>
            );

        case 'icon':
            const Icon = IconMap[component.name] || Info;
            return <Icon size={component.size || 24} />;

        case 'alert':
            const alertClasses = clsx('adk-ui-alert p-4 rounded-xl border mb-4 flex items-start gap-3 shadow-sm', {
                'adk-ui-alert--info': component.variant === 'info' || !component.variant,
                'adk-ui-alert--success': component.variant === 'success',
                'adk-ui-alert--warning': component.variant === 'warning',
                'adk-ui-alert--error': component.variant === 'error',
                'bg-blue-50 border-blue-200 text-blue-800': component.variant === 'info' || !component.variant,
                'bg-green-50 border-green-200 text-green-800': component.variant === 'success',
                'bg-yellow-50 border-yellow-200 text-yellow-800': component.variant === 'warning',
                'bg-red-50 border-red-200 text-red-800': component.variant === 'error',
            });
            const AlertIcon = component.variant === 'success' ? CheckCircle :
                component.variant === 'warning' ? AlertCircle :
                    component.variant === 'error' ? XCircle : Info;
            return (
                <div className={alertClasses}>
                    <AlertIcon className="w-5 h-5 mt-0.5" />
                    <div>
                        <div className="font-semibold">{component.title}</div>
                        {component.description && <div className="text-sm mt-1 opacity-90">{component.description}</div>}
                    </div>
                </div>
            );

        case 'card':
            const hasInputs = component.content.some(containsInput);

            const cardContent = (
                <>
                    {(component.title || component.description) && (
                        <div className="adk-ui-card__header px-5 py-4 border-b border-slate-200/80 dark:border-gray-700 bg-slate-50/80 dark:bg-gray-800">
                            {component.title && <h3 className="adk-ui-heading font-semibold text-lg tracking-tight text-slate-900 dark:text-white">{component.title}</h3>}
                            {component.description && <p className="text-gray-500 dark:text-gray-400 text-sm">{component.description}</p>}
                        </div>
                    )}
                    <div className="p-5">
                        {component.content.map((child, i) => <ComponentRenderer key={componentKey(child, i)} component={child} />)}
                    </div>
                    {component.footer && (
                        <div className="adk-ui-card__footer px-5 py-4 border-t border-slate-200/80 dark:border-gray-700 bg-slate-50/80 dark:bg-gray-800 flex flex-wrap gap-2 justify-end">
                            {component.footer.map((child, i) => <ComponentRenderer key={componentKey(child, i)} component={child} />)}
                        </div>
                    )}
                </>
            );

            return hasInputs ? (
                <form ref={cardFormRef} onSubmit={(event) => event.preventDefault()} className="adk-ui-card bg-white dark:bg-gray-900 rounded-xl border border-slate-200 dark:border-gray-700 shadow-sm overflow-hidden mb-4 min-w-0">
                    <FormContext.Provider value={{ onAction, formRef: cardFormRef }}>
                        {cardContent}
                    </FormContext.Provider>
                </form>
            ) : (
                <div className="adk-ui-card bg-white dark:bg-gray-900 rounded-xl border border-slate-200 dark:border-gray-700 shadow-sm overflow-hidden mb-4 min-w-0">
                    {cardContent}
                </div>
            );

        case 'stack':
            const stackClasses = clsx('flex', {
                'flex-col': component.direction === 'vertical',
                'flex-row': component.direction === 'horizontal',
            });
            return (
                <div className={stackClasses} style={{
                    gap: resolveGap(component.gap),
                    justifyContent: mapFlexPosition(component.justify),
                    alignItems: mapFlexAlign(component.align),
                    flexWrap: component.wrap ? 'wrap' : undefined,
                }}>
                    {component.children.map((child, i) => <ComponentRenderer key={componentKey(child, i)} component={child} />)}
                </div>
            );

        case 'text_input':
            const inputType = component.input_type || 'text';
            const textInputControlId = formControlId(component);
            return (
                <div className="mb-3">
                    <label htmlFor={textInputControlId} className="adk-ui-label block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{component.label}</label>
                    <input
                        id={textInputControlId}
                        type={inputType}
                        name={component.name}
                        placeholder={component.placeholder}
                        defaultValue={component.default_value}
                        required={component.required}
                        onChange={(event) => onAction?.({
                            action: 'input_change',
                            name: component.name,
                            value: event.currentTarget.value,
                        })}
                        className={clsx('adk-ui-control w-full px-3 py-2.5 border rounded-lg focus:ring-2 focus:ring-teal-600/25 focus:border-teal-700 outline-none bg-white dark:bg-gray-800 dark:border-gray-600 dark:text-white', {
                            'border-red-500 focus:ring-red-500 focus:border-red-500': component.error,
                        })}
                    />
                    {component.error && (
                        <p className="text-red-500 dark:text-red-400 text-sm mt-1">{component.error}</p>
                    )}
                </div>
            );

        case 'number_input':
            const numberInputControlId = formControlId(component);
            return (
                <div className="mb-3">
                    <label htmlFor={numberInputControlId} className="adk-ui-label block text-sm font-medium text-gray-700 mb-1">{component.label}</label>
                    <input
                        id={numberInputControlId}
                        type="number"
                        name={component.name}
                        min={component.min}
                        max={component.max}
                        step={component.step}
                        required={component.required}
                        onChange={(event) => {
                            const parsed = event.currentTarget.valueAsNumber;
                            onAction?.({
                                action: 'input_change',
                                name: component.name,
                                value: Number.isNaN(parsed) ? event.currentTarget.value : parsed,
                            });
                        }}
                        className={clsx('adk-ui-control w-full px-3 py-2.5 border rounded-lg focus:ring-2 focus:ring-teal-600/25 focus:border-teal-700 outline-none', {
                            'border-red-500 focus:ring-red-500 focus:border-red-500': component.error,
                        })}
                    />
                    {component.error && (
                        <p className="text-red-500 text-sm mt-1">{component.error}</p>
                    )}
                </div>
            );

        case 'select':
            const selectControlId = formControlId(component);
            return (
                <div className="mb-3">
                    <label htmlFor={selectControlId} className="adk-ui-label block text-sm font-medium text-gray-700 mb-1">{component.label}</label>
                    <select
                        id={selectControlId}
                        name={component.name}
                        required={component.required}
                        onChange={(event) => onAction?.({
                            action: 'input_change',
                            name: component.name,
                            value: event.currentTarget.value,
                        })}
                        className={clsx('adk-ui-control w-full px-3 py-2.5 border rounded-lg focus:ring-2 focus:ring-teal-600/25 focus:border-teal-700 outline-none', {
                            'border-red-500 focus:ring-red-500 focus:border-red-500': component.error,
                        })}
                    >
                        <option value="">Select...</option>
                        {component.options.map((opt) => (
                            <option key={opt.value} value={opt.value}>{opt.label}</option>
                        ))}
                    </select>
                    {component.error && (
                        <p className="text-red-500 text-sm mt-1">{component.error}</p>
                    )}
                </div>
            );

        case 'switch':
            const switchControlId = formControlId(component);
            return (
                <div className="mb-3 flex items-center">
                    <input
                        id={switchControlId}
                        type="checkbox"
                        name={component.name}
                        defaultChecked={component.default_checked}
                        onChange={(event) => onAction?.({
                            action: 'input_change',
                            name: component.name,
                            value: event.currentTarget.checked,
                        })}
                        className="adk-ui-control h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                    />
                    <label htmlFor={switchControlId} className="adk-ui-label ml-2 text-sm font-medium text-gray-700">{component.label}</label>
                </div>
            );

        case 'multi_select':
            const multiSelectControlId = formControlId(component);
            return (
                <div className="mb-3">
                    <label htmlFor={multiSelectControlId} className="adk-ui-label block text-sm font-medium text-gray-700 mb-1">{component.label}</label>
                    <select
                        id={multiSelectControlId}
                        name={component.name}
                        multiple
                        required={component.required}
                        size={Math.min(component.options.length, 5)}
                        onChange={(event) => {
                            const selected = Array.from(event.currentTarget.selectedOptions).map((opt) => opt.value);
                            onAction?.({
                                action: 'input_change',
                                name: component.name,
                                value: selected,
                            });
                        }}
                        className="adk-ui-control w-full px-3 py-2 border rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                    >
                        {component.options.map((opt, i) => (
                            <option key={i} value={opt.value}>{opt.label}</option>
                        ))}
                    </select>
                </div>
            );

        case 'date_input':
            const dateInputControlId = formControlId(component);
            return (
                <div className="mb-3">
                    <label htmlFor={dateInputControlId} className="adk-ui-label block text-sm font-medium text-gray-700 mb-1">{component.label}</label>
                    <input
                        id={dateInputControlId}
                        type="date"
                        name={component.name}
                        required={component.required}
                        onChange={(event) => onAction?.({
                            action: 'input_change',
                            name: component.name,
                            value: event.currentTarget.value,
                        })}
                        className="adk-ui-control w-full px-3 py-2 border rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                    />
                </div>
            );

        case 'slider':
            const sliderControlId = formControlId(component);
            return (
                <div className="mb-3">
                    <label htmlFor={sliderControlId} className="adk-ui-label block text-sm font-medium text-gray-700 mb-1">{component.label}</label>
                    <input
                        id={sliderControlId}
                        type="range"
                        name={component.name}
                        min={component.min}
                        max={component.max}
                        step={component.step}
                        defaultValue={component.default_value}
                        onChange={(event) => {
                            const parsed = event.currentTarget.valueAsNumber;
                            onAction?.({
                                action: 'input_change',
                                name: component.name,
                                value: Number.isNaN(parsed) ? event.currentTarget.value : parsed,
                            });
                        }}
                        className="adk-ui-control w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
                    />
                </div>
            );

        case 'progress':
            return (
                <div className="mb-3">
                    {component.label && <div className="adk-ui-caption text-sm text-gray-600 mb-1">{component.label}</div>}
                    <div className="adk-ui-progress__track w-full bg-gray-200 rounded-full h-2.5">
                        <div
                            className="adk-ui-progress__value bg-blue-600 h-2.5 rounded-full transition-all"
                            style={{ width: `${component.value}%` }}
                        />
                    </div>
                </div>
            );

        case 'textarea':
            const textareaControlId = formControlId(component);
            return (
                <div className="mb-3">
                    <label htmlFor={textareaControlId} className="adk-ui-label block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{component.label}</label>
                    <textarea
                        id={textareaControlId}
                        name={component.name}
                        placeholder={component.placeholder}
                        rows={component.rows || 4}
                        required={component.required}
                        defaultValue={component.default_value}
                        onChange={(event) => onAction?.({
                            action: 'input_change',
                            name: component.name,
                            value: event.currentTarget.value,
                        })}
                        className={clsx('adk-ui-control w-full px-3 py-2.5 border rounded-lg focus:ring-2 focus:ring-teal-600/25 focus:border-teal-700 outline-none bg-white dark:bg-gray-800 dark:border-gray-600 dark:text-white resize-y', {
                            'border-red-500 focus:ring-red-500 focus:border-red-500': component.error,
                        })}
                    />
                    {component.error && (
                        <p className="text-red-500 dark:text-red-400 text-sm mt-1">{component.error}</p>
                    )}
                </div>
            );

        case 'spinner':
            const spinnerSizes = {
                small: 'w-4 h-4',
                medium: 'w-8 h-8',
                large: 'w-12 h-12',
            };
            return (
                <div className="flex items-center gap-2">
                    <div className={clsx('animate-spin rounded-full border-2 border-blue-600 border-t-transparent', spinnerSizes[component.size || 'medium'])} />
                    {component.label && <span className="text-gray-600 dark:text-gray-400">{component.label}</span>}
                </div>
            );

        case 'skeleton':
            return (
                <div
                    className={clsx('animate-pulse bg-gray-200 dark:bg-gray-700', {
                        'h-4 rounded': component.variant === 'text' || !component.variant,
                        'rounded-full aspect-square': component.variant === 'circle',
                        'rounded': component.variant === 'rectangle',
                    })}
                    style={{ width: component.width || '100%', height: component.height }}
                />
            );

        case 'toast':
            const toastClasses = clsx('fixed bottom-4 right-4 p-4 rounded-lg shadow-lg flex items-center gap-3 z-50', {
                'bg-blue-50 border border-blue-200 text-blue-800': component.variant === 'info' || !component.variant,
                'bg-green-50 border border-green-200 text-green-800': component.variant === 'success',
                'bg-yellow-50 border border-yellow-200 text-yellow-800': component.variant === 'warning',
                'bg-red-50 border border-red-200 text-red-800': component.variant === 'error',
            });
            const ToastIcon = component.variant === 'success' ? CheckCircle :
                component.variant === 'warning' ? AlertCircle :
                    component.variant === 'error' ? XCircle : Info;
            return (
                <div className={toastClasses}>
                    <ToastIcon className="w-5 h-5" />
                    <span>{component.message}</span>
                    {component.dismissible !== false && (
                        <button
                            onClick={() => onAction?.({
                                action: 'button_click',
                                action_id: 'toast_dismiss',
                                source_component_id: component.id ?? 'toast_dismiss',
                            })}
                            className="ml-2 text-gray-500 hover:text-gray-700"
                        >
                            <XCircle className="w-4 h-4" />
                        </button>
                    )}
                </div>
            );

        case 'modal':
            const modalSizes = {
                small: 'max-w-sm',
                medium: 'max-w-lg',
                large: 'max-w-2xl',
                full: 'max-w-full mx-4',
            };
            return (
                <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
                    <div className={clsx('adk-ui-modal bg-white dark:bg-gray-900 rounded-lg border shadow-xl w-full', modalSizes[component.size || 'medium'])}>
                        <div className="p-4 border-b dark:border-gray-700 flex justify-between items-center">
                            <h3 className="adk-ui-heading font-semibold text-lg dark:text-white">{component.title}</h3>
                            {component.closable !== false && (
                                <button
                                    onClick={() => onAction?.({
                                        action: 'button_click',
                                        action_id: 'modal_close',
                                        source_component_id: component.id ?? 'modal_close',
                                    })}
                                    className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                                >
                                    <XCircle className="w-5 h-5" />
                                </button>
                            )}
                        </div>
                        <div className="p-4">
                                {component.content.map((child, i) => <ComponentRenderer key={componentKey(child, i)} component={child} />)}
                        </div>
                        {component.footer && (
                            <div className="p-4 border-t dark:border-gray-700 flex justify-end gap-2">
                                {component.footer.map((child, i) => <ComponentRenderer key={componentKey(child, i)} component={child} />)}
                            </div>
                        )}
                    </div>
                </div>
            );

        case 'grid':
            return (
                <div
                    className="grid gap-4 mb-4"
                    style={{
                        gridTemplateColumns: component.columns === 1
                            ? 'minmax(0, 1fr)'
                            : 'repeat(auto-fit, minmax(min(100%, 11rem), 1fr))',
                        gap: resolveGap(component.gap),
                    }}
                >
                    {component.children.map((child, i) => <ComponentRenderer key={componentKey(child, i)} component={child} />)}
                </div>
            );

        case 'list':
            return (
                <ul className="space-y-2 mb-4 list-disc list-inside">
                    {component.items.map((item, i) => (
                        <li key={i} className="adk-ui-body text-gray-700">{item}</li>
                    ))}
                </ul>
            );

        case 'key_value':
            const keyValuePairs = Array.isArray(boundData) ? boundData as NonNullable<typeof component.pairs> : (component.pairs ?? []);
            return (
                <dl className="grid grid-cols-2 gap-x-4 gap-y-2 mb-4">
                    {keyValuePairs.map((pair, i) => (
                        <React.Fragment key={i}>
                            <dt className="adk-ui-label font-medium text-gray-700">{pair.key}:</dt>
                            <dd className="adk-ui-body text-gray-900">{pair.value}</dd>
                        </React.Fragment>
                    ))}
                </dl>
            );

        case 'tabs':
            return (
                <div className="mb-4">
                    <div className="adk-ui-tabs border-b border-gray-200">
                        <nav className="flex space-x-4">
                            {component.tabs.map((tab, i) => (
                                <button
                                    key={i}
                                    onClick={() => {
                                        setActiveTab(i);
                                        onAction?.({ action: 'tab_change', index: i });
                                    }}
                                    className={clsx('px-4 py-2 border-b-2 font-medium text-sm transition-colors', {
                                        'adk-ui-tab--active border-blue-600 text-blue-600': activeTab === i,
                                        'border-transparent text-gray-500 hover:text-gray-700': activeTab !== i,
                                    })}
                                >
                                    {tab.label}
                                </button>
                            ))}
                        </nav>
                    </div>
                    <div className="p-4">
                        {component.tabs[activeTab].content.map((child, i) =>
                            <ComponentRenderer key={componentKey(child, i)} component={child} />
                        )}
                    </div>
                </div>
            );

        case 'table':
            // Table with sorting and pagination support
            const handleSort = (accessorKey: string) => {
                if (!component.sortable) return;
                if (sortColumn === accessorKey) {
                    setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
                } else {
                    setSortColumn(accessorKey);
                    setSortDirection('asc');
                }
            };

            const sourceRows = Array.isArray(boundData) ? boundData as Record<string, unknown>[] : (component.data ?? []);
            const tableData = [...sourceRows];
            if (sortColumn) {
                tableData.sort((a, b) => {
                    const aVal = a[sortColumn] ?? '';
                    const bVal = b[sortColumn] ?? '';
                    const cmp = String(aVal).localeCompare(String(bVal));
                    return sortDirection === 'asc' ? cmp : -cmp;
                });
            }

            const pageSize = component.page_size || tableData.length;
            const totalPages = Math.ceil(tableData.length / pageSize);
            const paginatedData = tableData.slice(currentPage * pageSize, (currentPage + 1) * pageSize);

            return (
                <div className="adk-ui-table mb-4 overflow-x-auto rounded-xl border border-slate-200 dark:border-gray-700 shadow-sm">
                    <table className={clsx('min-w-full divide-y divide-gray-200 dark:divide-gray-700 overflow-hidden')}>
                        <thead className="adk-ui-table__head bg-gray-50 dark:bg-gray-800">
                            <tr>
                                {component.columns.map((col, i) => (
                                    <th
                                        key={i}
                                        onClick={() => handleSort(col.accessor_key)}
                                        className={clsx(
                                            'px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider',
                                            component.sortable && col.sortable !== false && 'cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700'
                                        )}
                                    >
                                        {col.header}
                                        {sortColumn === col.accessor_key && (
                                            <span className="ml-1">{sortDirection === 'asc' ? '↑' : '↓'}</span>
                                        )}
                                    </th>
                                ))}
                            </tr>
                        </thead>
                        <tbody className="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                            {paginatedData.map((row, ri) => (
                                <tr key={ri} className={clsx(
                                    'hover:bg-gray-50 dark:hover:bg-gray-800',
                                    component.striped && ri % 2 === 1 && 'bg-gray-50 dark:bg-gray-800/50'
                                )}>
                                    {component.columns.map((col, ci) => (
                                        <td key={ci} className="px-4 py-3 text-sm text-gray-700 dark:text-gray-300">
                                            {String(row[col.accessor_key] ?? '')}
                                        </td>
                                    ))}
                                </tr>
                            ))}
                        </tbody>
                    </table>
                    {component.page_size && totalPages > 1 && (
                        <div className="flex items-center justify-between mt-2 px-2">
                            <span className="text-sm text-gray-500 dark:text-gray-400">
                                Page {currentPage + 1} of {totalPages}
                            </span>
                            <div className="flex gap-2">
                                <button
                                    onClick={() => setCurrentPage(Math.max(0, currentPage - 1))}
                                    disabled={currentPage === 0}
                                    className="px-3 py-1 text-sm border rounded hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50 dark:border-gray-600 dark:text-gray-300"
                                >
                                    Previous
                                </button>
                                <button
                                    onClick={() => setCurrentPage(Math.min(totalPages - 1, currentPage + 1))}
                                    disabled={currentPage === totalPages - 1}
                                    className="px-3 py-1 text-sm border rounded hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50 dark:border-gray-600 dark:text-gray-300"
                                >
                                    Next
                                </button>
                            </div>
                        </div>
                    )}
                </div>
            );

        case 'chart':
            const chartData = Array.isArray(boundData) ? boundData as Record<string, unknown>[] : (component.data ?? []);
            return (
                <OptionalRenderer
                    peerDependency="recharts"
                    capability="Chart"
                    loading={<div className="mb-4 h-[300px] animate-pulse rounded-xl bg-slate-100" aria-label="Loading chart" />}
                >
                    <ChartRenderer component={{ ...component, data: chartData }} />
                </OptionalRenderer>
            );

        case 'scene_3d':
            return (
                <OptionalRenderer
                    peerDependency="three"
                    capability="3D scene"
                    loading={<div className="adk-ui-scene-3d__loading" style={{ height: component.height ?? 440 }} aria-label="Loading 3D scene" />}
                >
                    <Scene3DRenderer component={component} onAction={onAction} />
                </OptionalRenderer>
            );

        case 'code_block':
            return (
                <div className="mb-4">
                    <pre className="adk-ui-code bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                        <code>{component.code}</code>
                    </pre>
                </div>
            );

        case 'image':
            const safeImageSrc = sanitizeUrl(resolveKitAssetSource(manifest, component.src), 'image');
            if (!safeImageSrc) {
                return renderBlockedAsset('image');
            }
            return (
                <div className="mb-4">
                    <img
                        src={safeImageSrc}
                        alt={component.alt || ''}
                        className="max-w-full h-auto rounded-lg"
                        loading="lazy"
                        referrerPolicy="no-referrer"
                    />
                </div>
            );

        case 'badge':
            const badgeClasses = clsx('adk-ui-badge inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium', {
                'adk-ui-badge--default': component.variant === 'default' || !component.variant,
                'adk-ui-badge--secondary': component.variant === 'secondary',
                'bg-gray-100 text-gray-800': component.variant === 'default' || !component.variant,
                'bg-blue-100 text-blue-800': component.variant === 'info',
                'bg-green-100 text-green-800': component.variant === 'success',
                'bg-yellow-100 text-yellow-800': component.variant === 'warning',
                'bg-red-100 text-red-800': component.variant === 'error',
                'bg-gray-200 text-gray-700': component.variant === 'secondary',
                'bg-transparent border border-gray-300 text-gray-700': component.variant === 'outline',
            });
            return <span className={badgeClasses}>{component.label}</span>;

        case 'divider':
            return <hr className="adk-ui-divider my-4 border-gray-200" />;

        case 'container':
            return (
                <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    {component.children.map((child, i) => <ComponentRenderer key={componentKey(child, i)} component={child} />)}
                </div>
            );

        default:
            return <div className="text-red-500 text-sm p-2 border border-red-200 rounded">Unknown component: {(component as any).type}</div>;
    }
});
