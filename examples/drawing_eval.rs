//! Scores drawings the way genui will judge them (`adk_ui::eval`).
//!
//! ```sh
//! # The sample drawings under fixtures/drawings (every one should be shown):
//! cargo run --example drawing_eval
//! # The built-in scenarios' prompts, one JSON object a line, to give a model:
//! cargo run --example drawing_eval -- --scenarios > prompts.jsonl
//! # A model's drawings, one {"scenario": name, "drawing": render_screen arguments} a line:
//! cargo run --example drawing_eval -- drawings.jsonl
//! ```
//!
//! It exits non-zero when a drawing is refused, so it can gate a prompt change in CI.

use std::path::{Path, PathBuf};

use adk_ui::catalogs;
use adk_ui::eval::{Report, scenarios, score};
use serde_json::{Value, json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argument = std::env::args().nth(1);
    let report = match argument.as_deref() {
        Some("--scenarios") => {
            for scenario in scenarios() {
                println!(
                    "{}",
                    json!({
                        "scenario": scenario.name,
                        "catalog": scenario.catalog.label(),
                        "prompt": scenario.prompt,
                        "expected": scenario.expected,
                    })
                );
            }
            return Ok(());
        }
        Some(path) => model_drawings(Path::new(path))?,
        None => samples()?,
    };
    println!("{report}");
    if report.rows.iter().any(|(_, score)| !score.accepted) {
        std::process::exit(1);
    }
    Ok(())
}

/// The sample drawings, each in its own catalog.
fn samples() -> Result<Report, Box<dyn std::error::Error>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/drawings");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    let mut report = Report::default();
    for path in paths {
        let fixture: Value = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
        let catalog = match fixture["catalog"].as_str() {
            Some("studio") => catalogs::studio(),
            Some("spatial") => catalogs::spatial(),
            _ => catalogs::insight(),
        };
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        report.add(name, score(catalog, &fixture["screen"], &[]));
    }
    Ok(report)
}

/// A model's drawings, each scored against the scenario it names.
fn model_drawings(path: &Path) -> Result<Report, Box<dyn std::error::Error>> {
    let known = scenarios();
    let mut report = Report::default();
    for (number, line) in std::fs::read_to_string(path)?.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let row: Value =
            serde_json::from_str(line).map_err(|error| format!("line {}: {error}", number + 1))?;
        let name = row["scenario"].as_str().unwrap_or("unnamed");
        let Some(scenario) = known.iter().find(|scenario| scenario.name == name) else {
            return Err(format!("line {}: no scenario {name}", number + 1).into());
        };
        report.add(
            format!("{name} #{}", number + 1),
            scenario.score(&row["drawing"]),
        );
    }
    Ok(report)
}
