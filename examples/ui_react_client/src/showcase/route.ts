export function resolveShowcaseExampleId(): string | null {
  return new URLSearchParams(window.location.search).get('example');
}
