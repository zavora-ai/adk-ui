//! What `show_3d_app` refuses before an app record is ever written.
//!
//! The home's own scan reads only the page template, never the code, which reaches the frame as
//! the call's input. So the server reads the code itself:
//! - a decision's words anywhere a person could read them;
//! - anything that reaches outside the frame.
//!
//! The scan is best effort: code that builds words or names at run time gets past it. The real
//! boundary is the sandbox, which has its own origin, `connect-src 'none'` and
//! `worker-src 'none'`, and whose tool calls are held for the person.

use std::sync::LazyLock;

use adk_surface::genui::reserved;
use regex::Regex;
use serde::Deserialize;

/// The longest code `show_3d_app` takes, in bytes.
pub const MAX_CODE_BYTES: usize = 32 * 1024;
/// The longest title, in characters.
pub const MAX_TITLE_CHARS: usize = 80;
/// The longest description, in characters.
pub const MAX_DESCRIPTION_CHARS: usize = 280;
/// The frame's height range, in CSS pixels. The home frames an app at most 640 tall.
pub const HEIGHTS: std::ops::RangeInclusive<u32> = 240..=640;
/// The height when the call names none.
pub const DEFAULT_HEIGHT: u32 = 420;

/// A `show_3d_app` call's arguments.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppInput {
    /// The app's title.
    pub title: String,
    /// One line on what it shows.
    #[serde(default)]
    pub description: Option<String>,
    /// The body of an async function given the app's kit.
    pub code: String,
    /// Its height in CSS pixels.
    #[serde(default)]
    pub height: Option<u32>,
}

/// String and template literals.
static LITERALS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""((?:[^"\\\n]|\\.)*)"|'((?:[^'\\\n]|\\.)*)'|`((?:[^`\\]|\\.)*)`"#)
        .expect("the literal pattern compiles")
});

/// What the code may not do, each with the reason the model reads.
static REACHES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    [
        (r"(?i)\b(?:https?|wss?|ftp)://", "it names a web address; apps here reach no sites"),
        (r"(?:^|[\s(=,'`\x22])//[A-Za-z0-9-]+\.[A-Za-z]", "it names a web address; apps here reach no sites"),
        (r"\bimport\s*\(", "it imports code; use THREE and addons, which it is given"),
        (r#"\bimport\b[^;\n]*\bfrom\b|^\s*import\s+["'{*]"#, "it imports code; use THREE and addons, which it is given"),
        (r"\bimportScripts\b", "it imports code; use THREE and addons, which it is given"),
        (r"\bfetch\s*\(|\bXMLHttpRequest\b|\bWebSocket\b|\bEventSource\b|\bsendBeacon\b|\bWebTransport\b", "it reaches the network; apps here reach no sites"),
        (r"\bpostMessage\b|\bwindow\s*\.\s*(?:parent|top|opener|frames)\b|\b(?:globalThis|self)\s*\.\s*(?:parent|top|opener)\b", "it talks to the page outside its frame"),
        (r"\beval\s*\(|\bnew\s+Function\b|\bFunction\s*\(", "it evaluates text as code"),
        (r"\bdocument\s*\.\s*(?:write|cookie|domain)\b|\blocalStorage\b|\bsessionStorage\b|\bindexedDB\b", "it reaches page storage or rewrites the page"),
        (r"\b(?:Worker|SharedWorker|ServiceWorker)\b|\bserviceWorker\b", "it starts a worker; apps here have none"),
    ]
    .into_iter()
    .map(|(pattern, reason)| (Regex::new(pattern).expect("a reach pattern compiles"), reason))
    .collect()
});

/// Why `input` is not shown, or `None` when it may be.
pub fn refusal(input: &AppInput) -> Option<String> {
    let title = input.title.trim();
    if title.is_empty() {
        return Some("give the app a title".into());
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Some(format!(
            "the title is longer than {MAX_TITLE_CHARS} characters"
        ));
    }
    if let Some(description) = &input.description
        && description.chars().count() > MAX_DESCRIPTION_CHARS
    {
        return Some(format!(
            "the description is longer than {MAX_DESCRIPTION_CHARS} characters"
        ));
    }
    if let Some(height) = input.height
        && !HEIGHTS.contains(&height)
    {
        return Some(format!(
            "height {height} is outside {}–{} pixels",
            HEIGHTS.start(),
            HEIGHTS.end()
        ));
    }
    if input.code.trim().is_empty() {
        return Some("the code is empty".into());
    }
    if input.code.len() > MAX_CODE_BYTES {
        return Some(format!(
            "the code is longer than {} KiB",
            MAX_CODE_BYTES / 1024
        ));
    }
    for text in [Some(title), input.description.as_deref()]
        .into_iter()
        .flatten()
    {
        if let Some(word) = reserved(text) {
            return Some(format!(
                "it shows “{word}”, a decision's words, which an app never shows"
            ));
        }
    }
    if let Some(word) = shown_words(&input.code) {
        return Some(format!(
            "it shows “{word}”, a decision's words, which an app never shows"
        ));
    }
    REACHES
        .iter()
        .find(|(pattern, _)| pattern.is_match(&input.code))
        .map(|(_, reason)| (*reason).to_string())
}

/// A decision's words in the code: in a literal, or anywhere on a line (comments included).
fn shown_words(code: &str) -> Option<&'static str> {
    for found in LITERALS.captures_iter(code) {
        let text = found
            .get(1)
            .or(found.get(2))
            .or(found.get(3))
            .map_or("", |m| m.as_str());
        if let Some(word) = reserved(&unescaped(text)) {
            return Some(word);
        }
    }
    code.lines().find_map(reserved)
}

/// A literal's text with its common escapes undone, so "Allow once" reads as it shows.
fn unescaped(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n' | 't' | 'r') => out.push(' '),
            Some('u') => {
                let braced = chars.peek() == Some(&'{');
                if braced {
                    chars.next();
                }
                let hex: String = std::iter::from_fn(|| {
                    chars.next_if(|c| c.is_ascii_hexdigit() || (braced && *c != '}'))
                })
                .take(if braced { 6 } else { 4 })
                .collect();
                if braced {
                    chars.next_if_eq(&'}');
                }
                if let Some(decoded) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    out.push(decoded);
                }
            }
            Some('x') => {
                let hex: String = chars.by_ref().take(2).collect();
                if let Some(decoded) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    out.push(decoded);
                }
            }
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(code: &str) -> AppInput {
        AppInput {
            title: "Solar system".into(),
            description: None,
            code: code.into(),
            height: None,
        }
    }

    #[test]
    fn plain_three_code_is_shown() {
        let code = "const sun = new THREE.Mesh(new THREE.SphereGeometry(1), \
                    new THREE.MeshStandardMaterial({ color: theme.palette[0] }));\n\
                    scene.add(sun);\nonFrame((dt) => { sun.rotation.y += dt; });\n\
                    label('Sun', sun.position);";
        assert_eq!(refusal(&input(code)), None);
    }

    #[test]
    fn decision_words_are_refused_however_they_are_written() {
        for code in [
            "label('Allow once', p)",
            "const b = `Always allow`;",
            "// Allow once\nscene.add(x)",
            "label(\"Allow\\u0020once\", p)",
            "label('Approve', p)",
        ] {
            assert!(
                refusal(&input(code)).is_some_and(|r| r.contains("decision")),
                "{code}"
            );
        }
        let mut titled = input("scene.add(x)");
        titled.title = "Approved".into();
        assert!(refusal(&titled).is_some());
    }

    #[test]
    fn reaching_outside_the_frame_is_refused() {
        for code in [
            "fetch('/x')",
            "const t = await import('three/addons/x.js')",
            "import { X } from 'three/addons/x.js'",
            "new WebSocket(u)",
            "window.parent.postMessage({}, '*')",
            "const u = 'https://example.com/a.glb'",
            "eval(text)",
            "new Function('a', b)",
            "localStorage.setItem('a', 1)",
            "new Worker(u)",
        ] {
            assert!(refusal(&input(code)).is_some(), "{code}");
        }
    }

    #[test]
    fn sizes_and_ranges_are_checked() {
        assert!(refusal(&input(&"x".repeat(MAX_CODE_BYTES + 1))).is_some());
        let mut tall = input("scene.add(x)");
        tall.height = Some(900);
        assert!(refusal(&tall).is_some());
        let mut untitled = input("scene.add(x)");
        untitled.title = "  ".into();
        assert!(refusal(&untitled).is_some());
    }

    #[test]
    fn everyday_words_pass() {
        let code = "const top = 2; mesh.position.set(0, top, 0); // stop spinning at the top\n\
                    const parentGroup = new THREE.Group();";
        assert_eq!(refusal(&input(code)), None);
    }
}
