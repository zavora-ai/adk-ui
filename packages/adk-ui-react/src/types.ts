export type Component =
    // Atoms
    | { type: 'text'; id?: string; content: string; variant?: TextVariant }
    | { type: 'button'; id?: string; label: string; action_id: string; variant?: ButtonVariant; disabled?: boolean; icon?: string }
    | { type: 'icon'; id?: string; name: string; size?: number }
    | { type: 'image'; id?: string; src: string; alt?: string }
    | { type: 'badge'; id?: string; label: string; variant?: BadgeVariant }
    // Inputs
    | { type: 'text_input'; id?: string; name: string; label: string; input_type?: 'text' | 'email' | 'password' | 'tel' | 'url'; placeholder?: string; required?: boolean; default_value?: string; min_length?: number; max_length?: number; error?: string }
    | { type: 'number_input'; id?: string; name: string; label: string; min?: number; max?: number; step?: number; required?: boolean; default_value?: number; error?: string }
    | { type: 'select'; id?: string; name: string; label: string; options: SelectOption[]; required?: boolean; error?: string }
    | { type: 'multi_select'; id?: string; name: string; label: string; options: SelectOption[]; required?: boolean }
    | { type: 'switch'; id?: string; name: string; label: string; default_checked?: boolean }
    | { type: 'date_input'; id?: string; name: string; label: string; required?: boolean }
    | { type: 'slider'; id?: string; name: string; label: string; min?: number; max?: number; step?: number; default_value?: number }
    | { type: 'textarea'; id?: string; name: string; label: string; placeholder?: string; rows?: number; required?: boolean; default_value?: string; error?: string }
    // Layouts
    | { type: 'stack'; id?: string; direction: 'horizontal' | 'vertical'; children: Component[]; gap?: number | string; justify?: string; align?: string; wrap?: boolean }
    | { type: 'grid'; id?: string; columns: number; children: Component[]; gap?: number | string }
    | { type: 'card'; id?: string; title?: string; description?: string; content: Component[]; footer?: Component[] }
    | { type: 'container'; id?: string; children: Component[]; padding?: number }
    | { type: 'divider'; id?: string }
    | { type: 'tabs'; id?: string; tabs: Tab[] }
    // Data Display
    | { type: 'table'; id?: string; columns: TableColumn[]; data?: Record<string, unknown>[]; data_source?: DataSource; sortable?: boolean; page_size?: number; striped?: boolean }
    | { type: 'list'; id?: string; items: string[]; ordered?: boolean }
    | { type: 'key_value'; id?: string; pairs?: KeyValuePair[]; data_source?: DataSource }
    | { type: 'code_block'; id?: string; code: string; language?: string }
    // Visualizations
    | { type: 'chart'; id?: string; title?: string; kind: ChartKind; data?: Record<string, unknown>[]; data_source?: DataSource; x_key: string; y_keys: string[]; x_type?: ChartXType; time_format?: string; window?: number; x_label?: string; y_label?: string; show_legend?: boolean; colors?: string[] }
    | Scene3dComponent
    // Feedback
    | { type: 'alert'; id?: string; title: string; description?: string; variant?: AlertVariant }
    | { type: 'progress'; id?: string; value: number; label?: string }
    | { type: 'toast'; id?: string; message: string; variant?: AlertVariant; duration?: number; dismissible?: boolean }
    | { type: 'modal'; id?: string; title: string; content: Component[]; footer?: Component[]; size?: ModalSize; closable?: boolean }
    | { type: 'spinner'; id?: string; size?: SpinnerSize; label?: string }
    | { type: 'skeleton'; id?: string; variant?: SkeletonVariant; width?: string; height?: string };

export type TextVariant = 'h1' | 'h2' | 'h3' | 'h4' | 'body' | 'caption' | 'code';
export type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost' | 'outline';
export type BadgeVariant = 'default' | 'info' | 'success' | 'warning' | 'error' | 'secondary' | 'outline';
export type AlertVariant = 'info' | 'success' | 'warning' | 'error';
export type ChartKind = 'bar' | 'line' | 'area' | 'pie' | 'scatter' | 'sparkline';
export type ChartXType = 'category' | 'numeric' | 'time';
export type DataRefresh = { mode: 'manual' } | { mode: 'interval'; secs: number } | { mode: 'push' };
export interface DataSource { binding_path: string; refresh?: DataRefresh; ttl?: number }
export type ModalSize = 'small' | 'medium' | 'large' | 'full';
export type SpinnerSize = 'small' | 'medium' | 'large';
export type SkeletonVariant = 'text' | 'circle' | 'rectangle';

export interface Scene3dComponent {
    type: 'scene_3d';
    id?: string;
    title?: string;
    description?: string;
    height?: number;
    background?: 'transparent' | 'surface' | 'muted' | 'contrast';
    camera?: {
        position?: [number, number, number];
        target?: [number, number, number];
        fov?: number;
    };
    objects: Scene3dObject[];
    auto_rotate?: boolean;
    controls?: boolean;
    fallback?: string;
}

export type Scene3dMaterialRole = 'primary' | 'secondary' | 'surface' | 'muted' | 'success' | 'warning' | 'info';
export type Scene3dObject =
    | {
        kind: 'primitive'; id: string; shape: 'box' | 'sphere' | 'cylinder' | 'torus' | 'plane';
        position?: [number, number, number]; rotation?: [number, number, number]; scale?: [number, number, number];
        material?: Scene3dMaterialRole; label?: string; action_id?: string;
    }
    | {
        kind: 'model'; id: string; asset_id: string;
        position?: [number, number, number]; rotation?: [number, number, number]; scale?: [number, number, number];
        label?: string; action_id?: string;
    };

export interface SelectOption {
    label: string;
    value: string;
}

export interface TableColumn {
    header: string;
    accessor_key: string;
    sortable?: boolean;
}

export interface Tab {
    label: string;
    content: Component[];
}

export interface KeyValuePair {
    key: string;
    value: string;
}

export interface UiResponse {
    id?: string;
    kit_id?: string;
    theme?: 'light' | 'dark' | 'system';
    components: Component[];
}

// --- User Events (UI → Agent) ---

export type UiEvent =
    | { action: 'form_submit'; action_id: string; data: Record<string, unknown>; source_component_id?: string }
    | { action: 'button_click'; action_id: string; source_component_id?: string }
    | { action: 'input_change'; name: string; value: unknown }
    | { action: 'tab_change'; index: number };

export interface SurfaceRef { id: string; version: number }
export interface UiEventEnvelope {
    surface_ref: SurfaceRef;
    event: UiEvent;
    occurred_at: string;
    metadata?: Record<string, unknown>;
}

export function uiEventToMessage(event: UiEvent): string {
    switch (event.action) {
        case 'form_submit':
            return `[UI Event: Form submitted]\nAction: ${event.action_id}\nData:\n${JSON.stringify(event.data, null, 2)}`;
        case 'button_click':
            return `[UI Event: Button clicked]\nAction: ${event.action_id}`;
        case 'input_change':
            return `[UI Event: Input changed]\nField: ${event.name}\nValue: ${event.value}`;
        case 'tab_change':
            return `[UI Event: Tab changed]\nIndex: ${event.index}`;
    }
}

// --- Streaming Updates (Agent → UI) ---

export type UiOperation = 'replace' | 'patch' | 'append' | 'remove';

export interface UiUpdate {
    target_id: string;
    operation: UiOperation;
    payload?: Component;
}
