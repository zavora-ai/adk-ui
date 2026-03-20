import type { Component, UiUpdate } from './types';

export function applyUiUpdates(component: Component, updates: UiUpdate[]): Component | null {
    return updates.reduce<Component | null>((current, update) => {
        if (!current) return null;
        return applyUiUpdate(current, update);
    }, component);
}

export function applyUiUpdate(component: Component, update: UiUpdate): Component | null {
    if (component.id === update.target_id) {
        return applyUpdateToTarget(component, update);
    }

    switch (component.type) {
        case 'stack': {
            const updated = applyToChildren(component.children, update);
            if (!updated.changed) return component;
            return { ...component, children: updated.children };
        }
        case 'grid': {
            const updated = applyToChildren(component.children, update);
            if (!updated.changed) return component;
            return { ...component, children: updated.children };
        }
        case 'container': {
            const updated = applyToChildren(component.children, update);
            if (!updated.changed) return component;
            return { ...component, children: updated.children };
        }
        case 'card': {
            const contentUpdate = applyToChildren(component.content, update);
            const footerUpdate = component.footer ? applyToChildren(component.footer, update) : { children: component.footer, changed: false };
            if (!contentUpdate.changed && !footerUpdate.changed) return component;
            return {
                ...component,
                content: contentUpdate.children,
                footer: footerUpdate.children,
            };
        }
        case 'tabs': {
            let changed = false;
            const tabs = component.tabs.map((tab) => {
                const updated = applyToChildren(tab.content, update);
                if (updated.changed) {
                    changed = true;
                    return { ...tab, content: updated.children };
                }
                return tab;
            });
            if (!changed) return component;
            return { ...component, tabs };
        }
        case 'modal': {
            const contentUpdate = applyToChildren(component.content, update);
            const footerUpdate = component.footer ? applyToChildren(component.footer, update) : { children: component.footer, changed: false };
            if (!contentUpdate.changed && !footerUpdate.changed) return component;
            return {
                ...component,
                content: contentUpdate.children,
                footer: footerUpdate.children,
            };
        }
        default:
            return component;
    }
}

function applyUpdateToTarget(component: Component, update: UiUpdate): Component | null {
    switch (update.operation) {
        case 'remove':
            return null;
        case 'replace':
            return update.payload ?? component;
        case 'patch':
            if (!update.payload) return component;
            return {
                ...component,
                ...update.payload,
                id: update.payload.id ?? component.id,
            } as Component;
        case 'append':
            if (!update.payload) return component;
            return appendChild(component, update.payload);
        default:
            return component;
    }
}

function appendChild(component: Component, child: Component): Component {
    switch (component.type) {
        case 'stack':
            return { ...component, children: [...component.children, child] };
        case 'grid':
            return { ...component, children: [...component.children, child] };
        case 'container':
            return { ...component, children: [...component.children, child] };
        case 'card':
            return { ...component, content: [...component.content, child] };
        case 'tabs':
            return component;
        case 'modal':
            return { ...component, content: [...component.content, child] };
        default:
            return component;
    }
}

function applyToChildren(children: Component[], update: UiUpdate) {
    let changed = false;
    const next = children.flatMap((child) => {
        const updated = applyUiUpdate(child, update);
        if (!updated) {
            changed = true;
            return [];
        }
        if (updated !== child) {
            changed = true;
        }
        return [updated];
    });
    return { children: next, changed };
}
