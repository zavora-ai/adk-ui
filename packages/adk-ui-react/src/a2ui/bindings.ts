export type DataBinding = { path: string };
export type FunctionCall = {
    call: string;
    args?: unknown[] | Record<string, unknown>;
    returnType?: string;
};

export type CheckRule = Record<string, unknown> & {
    call?: string;
    args?: unknown[] | Record<string, unknown>;
    message: string;
};

export interface ResolveContext {
    dataModel: Record<string, unknown>;
    scope?: Record<string, unknown>;
    functions?: FunctionRegistry;
}

export type FunctionRegistry = Record<string, (args: unknown[], ctx: ResolveContext) => unknown>;

const DEFAULT_FUNCTIONS: FunctionRegistry = {
    now: () => new Date().toISOString(),
    concat: (args) => args.map((value) => stringifyValue(value)).join(''),
    add: (args) => args.reduce<number>((total, value) => total + toNumber(value), 0),
    required: (args) => isPresent(readNamedArgument(args, 'value', 0)),
    regex: (args) => {
        const value = stringifyValue(readNamedArgument(args, 'value', 0));
        const pattern = stringifyValue(readNamedArgument(args, 'pattern', 1));
        const flags = stringifyValue(readNamedArgument(args, 'flags', 2));
        if (!pattern) {
            return false;
        }
        try {
            return new RegExp(pattern, flags).test(value);
        } catch {
            return false;
        }
    },
    length: (args) => {
        const value = readNamedArgument(args, 'value', 0);
        const config = extractConstraintArgs(args);
        const size = stringifyValue(value).length;
        const min = toOptionalNumber(config.min);
        const max = toOptionalNumber(config.max);
        if (min !== undefined && size < min) {
            return false;
        }
        if (max !== undefined && size > max) {
            return false;
        }
        return min !== undefined || max !== undefined;
    },
    numeric: (args) => {
        const value = readNamedArgument(args, 'value', 0);
        const parsed = Number(value);
        if (Number.isNaN(parsed)) {
            return false;
        }
        const config = extractConstraintArgs(args);
        const min = toOptionalNumber(config.min);
        const max = toOptionalNumber(config.max);
        if (min !== undefined && parsed < min) {
            return false;
        }
        if (max !== undefined && parsed > max) {
            return false;
        }
        return true;
    },
    email: (args) => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(
        stringifyValue(readNamedArgument(args, 'value', 0)).trim(),
    ),
    formatString: (args, ctx) => formatString(String(args[0] ?? ''), ctx),
    formatNumber: (args) => {
        const value = toNumber(readNamedArgument(args, 'value', 0));
        const decimals = toOptionalNumber(readNamedArgument(args, 'decimals', 1));
        const useGrouping = toOptionalBoolean(readNamedArgument(args, 'useGrouping', 2)) ?? true;
        return new Intl.NumberFormat(undefined, {
            minimumFractionDigits: decimals ?? 0,
            maximumFractionDigits: decimals ?? (Number.isInteger(value) ? 0 : 2),
            useGrouping,
        }).format(value);
    },
    formatCurrency: (args) => {
        const value = toNumber(readNamedArgument(args, 'value', 0));
        const currency = stringifyValue(readNamedArgument(args, 'currency', 1) ?? 'USD').toUpperCase();
        try {
            return new Intl.NumberFormat(undefined, {
                style: 'currency',
                currency,
            }).format(value);
        } catch {
            return `${currency} ${value}`;
        }
    },
    formatDate: (args) => {
        const date = parseDate(readNamedArgument(args, 'value', 0));
        const format = stringifyValue(readNamedArgument(args, 'format', 1));
        if (!date) {
            return '';
        }
        return formatDateValue(date, format);
    },
    pluralize: (args) => {
        const count = toNumber(readNamedArgument(args, 'count', 0));
        const variants = asRecord(readNamedArgument(args, 'variants', 1))
            ?? asRecord(readNamedArgument(args, 'options', 1))
            ?? {};
        const category = new Intl.PluralRules().select(count);
        const selected = variants[category] ?? variants.other;
        return stringifyValue(selected).replace(/\{count\}/g, String(count));
    },
    openUrl: (args) => stringifyValue(readNamedArgument(args, 'url', 0)),
    and: (args) => flattenBooleanArgs(args).every(Boolean),
    or: (args) => flattenBooleanArgs(args).some(Boolean),
    not: (args) => !Boolean(readNamedArgument(args, 'value', 0)),
};

export function isDataBinding(value: unknown): value is DataBinding {
    return (
        typeof value === 'object'
        && value !== null
        && 'path' in value
        && typeof (value as { path?: unknown }).path === 'string'
        && Object.keys(value as object).length === 1
    );
}

export function isFunctionCall(value: unknown): value is FunctionCall {
    return (
        typeof value === 'object'
        && value !== null
        && 'call' in value
        && typeof (value as { call?: unknown }).call === 'string'
    );
}

export function resolvePath(
    dataModel: Record<string, unknown>,
    path: string,
    scope?: Record<string, unknown>,
): unknown {
    const source = path.startsWith('/') ? dataModel : (scope ?? dataModel);
    if (path === '/' || path.length === 0) {
        return source;
    }
    const tokens = path.replace(/^\//, '').split('/').filter(Boolean);
    let cursor: unknown = source;
    for (const token of tokens) {
        if (typeof cursor !== 'object' || cursor === null) {
            return undefined;
        }
        cursor = (cursor as Record<string, unknown>)[token];
    }
    return cursor;
}

export function resolveDynamicValue(
    value: unknown,
    dataModel: Record<string, unknown>,
    scope?: Record<string, unknown>,
    functions?: FunctionRegistry,
): unknown {
    if (isDataBinding(value)) {
        return resolvePath(dataModel, value.path, scope);
    }
    if (isFunctionCall(value)) {
        return evaluateFunctionCall(value, { dataModel, scope, functions });
    }
    return value;
}

export function evaluateChecks(
    checks: unknown,
    dataModel: Record<string, unknown>,
    scope?: Record<string, unknown>,
    functions?: FunctionRegistry,
): { valid: boolean; message?: string; failedCheck?: CheckRule } {
    if (!Array.isArray(checks) || checks.length === 0) {
        return { valid: true };
    }

    for (const check of checks) {
        if (!isCheckRule(check)) {
            continue;
        }
        const passed = evaluateLogicExpression(check, dataModel, scope, functions);
        if (!passed) {
            return {
                valid: false,
                message: check.message,
                failedCheck: check,
            };
        }
    }

    return { valid: true };
}

export function resolveDynamicString(
    value: unknown,
    dataModel: Record<string, unknown>,
    scope?: Record<string, unknown>,
    functions?: FunctionRegistry,
): string {
    const resolved = resolveDynamicValue(value, dataModel, scope, functions);
    return stringifyValue(resolved);
}

function evaluateFunctionCall(call: FunctionCall, ctx: ResolveContext): unknown {
    const registry = { ...DEFAULT_FUNCTIONS, ...(ctx.functions ?? {}) };
    const fn = registry[call.call];
    if (!fn) {
        return undefined;
    }
    const args = resolveFunctionArgs(call.args, ctx);
    return fn(args, ctx);
}

function resolveFunctionArgs(args: FunctionCall['args'], ctx: ResolveContext): unknown[] {
    if (Array.isArray(args)) {
        return args.map((arg) =>
            resolveDynamicValue(arg, ctx.dataModel, ctx.scope, ctx.functions),
        );
    }

    const namedArgs = asRecord(args);
    if (namedArgs) {
        const resolvedEntries = Object.entries(namedArgs).map(([key, value]) => [
            key,
            resolveDynamicValue(value, ctx.dataModel, ctx.scope, ctx.functions),
        ]);
        return [Object.fromEntries(resolvedEntries)];
    }

    return [];
}

function formatString(template: string, ctx: ResolveContext): string {
    let output = '';
    let index = 0;
    while (index < template.length) {
        if (template[index] === '\\' && template[index + 1] === '$' && template[index + 2] === '{') {
            output += '${';
            index += 3;
            continue;
        }
        if (template[index] === '$' && template[index + 1] === '{') {
            const { expression, nextIndex } = parseExpression(template, index + 2);
            const value = resolveExpression(expression, ctx);
            output += stringifyValue(value);
            index = nextIndex + 1;
            continue;
        }
        output += template[index];
        index += 1;
    }
    return output;
}

function parseExpression(source: string, startIndex: number): { expression: string; nextIndex: number } {
    let index = startIndex;
    let depth = 1;
    let inString: '"' | "'" | null = null;
    while (index < source.length) {
        const char = source[index];
        if (inString) {
            if (char === '\\') {
                index += 2;
                continue;
            }
            if (char === inString) {
                inString = null;
            }
            index += 1;
            continue;
        }
        if (char === '"' || char === "'") {
            inString = char;
            index += 1;
            continue;
        }
        if (char === '$' && source[index + 1] === '{') {
            depth += 1;
            index += 2;
            continue;
        }
        if (char === '}') {
            depth -= 1;
            if (depth === 0) {
                return { expression: source.slice(startIndex, index), nextIndex: index };
            }
        }
        index += 1;
    }
    return { expression: source.slice(startIndex), nextIndex: source.length - 1 };
}

function resolveExpression(expression: string, ctx: ResolveContext): unknown {
    const trimmed = expression.trim();
    if (trimmed.startsWith('/')) {
        return resolvePath(ctx.dataModel, trimmed, ctx.scope);
    }
    if (trimmed.length === 0) {
        return '';
    }
    const callMatch = /^([a-zA-Z_][\w]*)\((.*)\)$/.exec(trimmed);
    if (callMatch) {
        const [, name, rawArgs] = callMatch;
        const args = splitArgs(rawArgs).map((arg) => resolveArgument(arg, ctx));
        return evaluateFunctionCall({ call: name, args }, ctx);
    }
    return resolvePath(ctx.dataModel, trimmed, ctx.scope);
}

function splitArgs(raw: string): string[] {
    const args: string[] = [];
    let current = '';
    let depth = 0;
    let inString: '"' | "'" | null = null;
    for (let index = 0; index < raw.length; index += 1) {
        const char = raw[index];
        if (inString) {
            current += char;
            if (char === '\\') {
                current += raw[index + 1] ?? '';
                index += 1;
                continue;
            }
            if (char === inString) {
                inString = null;
            }
            continue;
        }
        if (char === '"' || char === "'") {
            inString = char;
            current += char;
            continue;
        }
        if (char === '(') {
            depth += 1;
            current += char;
            continue;
        }
        if (char === ')') {
            depth = Math.max(0, depth - 1);
            current += char;
            continue;
        }
        if (char === ',' && depth === 0) {
            args.push(current.trim());
            current = '';
            continue;
        }
        current += char;
    }
    if (current.trim().length > 0) {
        args.push(current.trim());
    }
    return args;
}

function resolveArgument(raw: string, ctx: ResolveContext): unknown {
    const trimmed = raw.trim();
    if (trimmed.startsWith('${') && trimmed.endsWith('}')) {
        return resolveExpression(trimmed.slice(2, -1), ctx);
    }
    if (trimmed.startsWith('/') || trimmed.match(/^[a-zA-Z_]/)) {
        const resolved = resolveExpression(trimmed, ctx);
        if (resolved !== undefined) {
            return resolved;
        }
    }
    if ((trimmed.startsWith('"') && trimmed.endsWith('"'))
        || (trimmed.startsWith("'") && trimmed.endsWith("'"))) {
        return unquote(trimmed);
    }
    if (trimmed === 'true') {
        return true;
    }
    if (trimmed === 'false') {
        return false;
    }
    if (trimmed === 'null') {
        return null;
    }
    if (trimmed.length === 0) {
        return undefined;
    }
    const numeric = Number(trimmed);
    if (!Number.isNaN(numeric)) {
        return numeric;
    }
    return trimmed;
}

function evaluateLogicExpression(
    expression: unknown,
    dataModel: Record<string, unknown>,
    scope?: Record<string, unknown>,
    functions?: FunctionRegistry,
): boolean {
    if (typeof expression === 'boolean') {
        return expression;
    }

    if (isFunctionCall(expression)) {
        return Boolean(evaluateFunctionCall(expression, { dataModel, scope, functions }));
    }

    const record = asRecord(expression);
    if (!record) {
        return Boolean(resolveDynamicValue(expression, dataModel, scope, functions));
    }

    if (Array.isArray(record.and)) {
        return record.and.every((entry) =>
            evaluateLogicExpression(entry, dataModel, scope, functions),
        );
    }

    if (Array.isArray(record.or)) {
        return record.or.some((entry) =>
            evaluateLogicExpression(entry, dataModel, scope, functions),
        );
    }

    if ('not' in record) {
        return !evaluateLogicExpression(record.not, dataModel, scope, functions);
    }

    if (record.true === true) {
        return true;
    }

    if (record.false === false) {
        return false;
    }

    return Boolean(resolveDynamicValue(record, dataModel, scope, functions));
}

function isCheckRule(value: unknown): value is CheckRule {
    return typeof value === 'object'
        && value !== null
        && typeof (value as { message?: unknown }).message === 'string';
}

function stringifyValue(value: unknown): string {
    if (value === null || typeof value === 'undefined') {
        return '';
    }
    if (typeof value === 'string') {
        return value;
    }
    if (typeof value === 'number' || typeof value === 'boolean') {
        return String(value);
    }
    return JSON.stringify(value);
}

function toNumber(value: unknown): number {
    const numeric = Number(value);
    return Number.isNaN(numeric) ? 0 : numeric;
}

function toOptionalNumber(value: unknown): number | undefined {
    if (value === null || typeof value === 'undefined' || value === '') {
        return undefined;
    }
    const numeric = Number(value);
    return Number.isNaN(numeric) ? undefined : numeric;
}

function toOptionalBoolean(value: unknown): boolean | undefined {
    if (typeof value === 'boolean') {
        return value;
    }
    return undefined;
}

function unquote(value: string): string {
    return value.slice(1, -1).replace(/\\(["'])/g, '$1');
}

function readNamedArgument(args: unknown[], key: string, fallbackIndex: number): unknown {
    const named = extractNamedArgs(args);
    if (named && key && key in named) {
        return named[key];
    }
    return args[fallbackIndex];
}

function extractNamedArgs(args: unknown[]): Record<string, unknown> | null {
    if (args.length !== 1) {
        return null;
    }
    return asRecord(args[0]);
}

function extractConstraintArgs(args: unknown[]): Record<string, unknown> {
    const named = extractNamedArgs(args);
    if (named && ('min' in named || 'max' in named)) {
        return named;
    }
    return asRecord(args[1]) ?? {};
}

function flattenBooleanArgs(args: unknown[]): boolean[] {
    const named = extractNamedArgs(args);
    if (named) {
        if (Array.isArray(named.values)) {
            return named.values.map(Boolean);
        }
        return Object.values(named).map(Boolean);
    }
    return args.map(Boolean);
}

function isPresent(value: unknown): boolean {
    if (value === null || typeof value === 'undefined') {
        return false;
    }
    if (typeof value === 'string') {
        return value.trim().length > 0;
    }
    if (Array.isArray(value)) {
        return value.length > 0;
    }
    return true;
}

function asRecord(value: unknown): Record<string, unknown> | null {
    return typeof value === 'object' && value !== null && !Array.isArray(value)
        ? value as Record<string, unknown>
        : null;
}

function parseDate(value: unknown): Date | null {
    if (value instanceof Date && Number.isFinite(value.getTime())) {
        return value;
    }
    const date = new Date(String(value));
    return Number.isFinite(date.getTime()) ? date : null;
}

function formatDateValue(date: Date, format: string): string {
    if (!format) {
        return date.toISOString();
    }

    const tokenMap: Record<string, string> = {
        YYYY: String(date.getFullYear()),
        yyyy: String(date.getFullYear()),
        MMMM: new Intl.DateTimeFormat(undefined, { month: 'long' }).format(date),
        MMM: new Intl.DateTimeFormat(undefined, { month: 'short' }).format(date),
        MM: String(date.getMonth() + 1).padStart(2, '0'),
        M: String(date.getMonth() + 1),
        dd: String(date.getDate()).padStart(2, '0'),
        d: String(date.getDate()),
        EEEE: new Intl.DateTimeFormat(undefined, { weekday: 'long' }).format(date),
        E: new Intl.DateTimeFormat(undefined, { weekday: 'short' }).format(date),
        HH: String(date.getHours()).padStart(2, '0'),
        H: String(date.getHours()),
        hh: String(toTwelveHour(date.getHours())).padStart(2, '0'),
        h: String(toTwelveHour(date.getHours())),
        mm: String(date.getMinutes()).padStart(2, '0'),
        ss: String(date.getSeconds()).padStart(2, '0'),
        a: date.getHours() >= 12 ? 'PM' : 'AM',
    };

    return format.replace(/YYYY|yyyy|MMMM|MMM|MM|M|dd|d|EEEE|E|HH|H|hh|h|mm|ss|a/g, (token) => (
        tokenMap[token] ?? token
    ));
}

function toTwelveHour(hour: number): number {
    const normalized = hour % 12;
    return normalized === 0 ? 12 : normalized;
}
