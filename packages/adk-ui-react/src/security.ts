const ABSOLUTE_SCHEME_RE = /^[a-zA-Z][a-zA-Z\d+.-]*:/;
const LOOPBACK_HOSTS = new Set(['localhost', '127.0.0.1', '[::1]', '::1']);
const SAFE_DATA_IMAGE_RE = /^data:image\/(?:png|jpeg|jpg|gif|webp|avif);/i;

export type SafeUrlKind = 'anchor' | 'image' | 'media';

function isRelativeUrl(value: string): boolean {
    return !ABSOLUTE_SCHEME_RE.test(value) && !value.startsWith('//');
}

function isAllowedHttpUrl(parsed: URL): boolean {
    if (parsed.protocol === 'https:') {
        return true;
    }

    return parsed.protocol === 'http:' && LOOPBACK_HOSTS.has(parsed.hostname.toLowerCase());
}

export function sanitizeUrl(raw: string | null | undefined, kind: SafeUrlKind): string | null {
    const value = raw?.trim();
    if (!value) {
        return null;
    }

    if (isRelativeUrl(value)) {
        return value;
    }

    let parsed: URL;
    try {
        parsed = new URL(value);
    } catch {
        return null;
    }

    if (isAllowedHttpUrl(parsed)) {
        return parsed.toString();
    }

    if (kind === 'anchor' && (parsed.protocol === 'mailto:' || parsed.protocol === 'tel:')) {
        return parsed.toString();
    }

    if ((kind === 'image' || kind === 'media') && parsed.protocol === 'blob:') {
        return parsed.toString();
    }

    if (kind === 'image' && parsed.protocol === 'data:' && SAFE_DATA_IMAGE_RE.test(value)) {
        return value;
    }

    return null;
}

export function isExternalNavigationUrl(url: string): boolean {
    return url.startsWith('http://') || url.startsWith('https://');
}
