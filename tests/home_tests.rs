//! adk-ui end to end on ADK-Rust: a harness agent draws the six sample drawings with
//! `genui`'s render tools in adk-ui's studio catalog, a home that registers adk-ui's catalogs
//! shows every one (charts natively, extension components as their home views), and a form
//! inside a drawing is held for the person with only its fields. The projected drawings are
//! what `examples/custom_host` shows offline (`examples/custom_host/public/surfaces.json`,
//! written with `ADK_UI_WRITE_SURFACES=1`).

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use adk_core::{
    ApprovalDecision, ApprovalRequest, ApprovalService, Event, InMemoryApprovalStore,
    RequestedCall, SessionId, StoreApprovalService, UserId,
};
use adk_harness::{Harness, HarnessAgent, InstructionsConfig};
use adk_model::{ScriptedLlm, ScriptedResponse};
use adk_runner::Runner;
use adk_session::{InMemorySessionService, SessionService};
use adk_surface::genui::{ChartKind, DrawnNode, DrawnSurface};
use adk_surface::{
    Caller, Command, CommandOutcome, LocalSource, Origin, Source, SurfaceEngine, SurfaceKind,
    ThreadItem,
};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use futures::StreamExt;
use serde_json::{Map, Value, json};

const APP: &str = "home";
const SESSION: &str = "thread-1";

/// A source over the session the agent drew in, keeping every call asked for, never running it.
struct Sending {
    inner: LocalSource,
    busy: AtomicBool,
    asked: Mutex<Vec<RequestedCall>>,
}

#[async_trait]
impl Source for Sending {
    fn app_name(&self) -> &str {
        self.inner.app_name()
    }

    async fn transcript(
        &self,
        user: &str,
        session: &str,
    ) -> adk_surface::Result<Option<Vec<Event>>> {
        self.inner.transcript(user, session).await
    }

    async fn requests(
        &self,
        caller: &Caller,
        session: Option<&str>,
        open_only: bool,
    ) -> adk_surface::Result<Vec<ApprovalRequest>> {
        self.inner.requests(caller, session, open_only).await
    }

    async fn request(&self, id: &str) -> adk_surface::Result<Option<ApprovalRequest>> {
        self.inner.request(id).await
    }

    async fn decide(
        &self,
        id: &str,
        expected_version: u64,
        decision: ApprovalDecision,
    ) -> adk_surface::Result<ApprovalRequest> {
        self.inner.decide(id, expected_version, decision).await
    }

    async fn busy(&self, _user: &str, _session: &str) -> adk_surface::Result<bool> {
        Ok(self.busy.load(Ordering::SeqCst))
    }

    async fn request_call(
        &self,
        _caller: &Caller,
        _session: &str,
        _text: &str,
        call: RequestedCall,
    ) -> adk_surface::Result<()> {
        self.asked.lock().unwrap().push(call);
        Ok(())
    }
}

/// A tool a drawing may ask for; a send asks the person first, so it never runs here.
struct Asked(&'static str);

#[async_trait]
impl adk_core::Tool for Asked {
    fn name(&self) -> &str {
        self.0
    }

    fn description(&self) -> &str {
        "A tool a drawing asks for."
    }

    async fn execute(
        &self,
        _: Arc<dyn adk_core::ToolContext>,
        _: Value,
    ) -> adk_core::Result<Value> {
        panic!("{} never runs here: a send asks the person first", self.0);
    }
}

struct Home {
    engine: SurfaceEngine,
    source: Arc<Sending>,
    _dir: tempfile::TempDir,
}

/// The agent draws every sample drawing, and the throughput one's patches, in one turn.
async fn home() -> Home {
    let dir = tempfile::tempdir().unwrap();
    let sessions: Arc<dyn SessionService> = Arc::new(InMemorySessionService::new());
    let mut script = Vec::new();
    for fixture in common::fixtures() {
        script.push(ScriptedResponse::function_call(
            "render_screen",
            fixture.screen.clone(),
        ));
        for patch in &fixture.patches {
            script.push(ScriptedResponse::function_call(
                "patch_surface",
                patch.clone(),
            ));
        }
    }
    script.push(ScriptedResponse::text("Here is where things stand."));
    let agent = HarnessAgent::builder("operations")
        .model(Arc::new(ScriptedLlm::new("scripted", script)))
        .workspace(adk_devtools::Workspace::new(dir.path()))
        .project_instructions(InstructionsConfig {
            user_file: None,
            require_trust: false,
            ..InstructionsConfig::default()
        })
        .without_compaction()
        .toolset(Arc::new(adk_ui::toolset_for(adk_ui::catalogs::studio())))
        .tool(Arc::new(Asked("start_rollback")))
        .tool(Arc::new(Asked("open_ticket")))
        .tool(Arc::new(Asked("reserve_dock")))
        .build()
        .unwrap();
    let agent = Arc::new(agent);
    let runner = agent
        .configure_runner(
            Runner::builder()
                .app_name(APP)
                .agent(agent.clone())
                .session_service(sessions.clone()),
        )
        .build()
        .unwrap();
    let harness = Harness::with_runner(agent, runner).unwrap();
    let user = UserId::new("ana").unwrap();
    let session = harness
        .open_session(&user, Some(&SessionId::new(SESSION).unwrap()))
        .await
        .unwrap();
    let stream = harness
        .send(&user, &session, "how is checkout doing?")
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(20), stream.collect::<Vec<_>>())
        .await
        .expect("the drawing turn finishes");

    let approvals: Arc<dyn ApprovalService> = Arc::new(StoreApprovalService::new(Arc::new(
        InMemoryApprovalStore::new(),
    )));
    let inner = LocalSource::builder(APP)
        .sessions(sessions)
        .approvals(approvals)
        .build()
        .unwrap();
    let source = Arc::new(Sending {
        inner,
        busy: AtomicBool::new(false),
        asked: Mutex::new(Vec::new()),
    });
    let engine = adk_ui::catalogs::register(SurfaceEngine::builder(
        Arc::clone(&source) as Arc<dyn Source>
    ))
    .agent_name(APP, "Operations")
    .utc_offset(chrono::FixedOffset::east_opt(0).unwrap())
    .build();
    Home {
        engine,
        source,
        _dir: dir,
    }
}

fn web() -> Caller {
    Caller::human("ana", SurfaceKind::Web, Origin::Pointer).with_groups(["owner"])
}

async fn items(home: &Home) -> Vec<ThreadItem> {
    home.engine
        .thread(&web(), SESSION)
        .await
        .unwrap()
        .into_iter()
        .flat_map(|entry| entry.items)
        .collect()
}

/// The latest version of every drawn surface, by surface id.
fn latest(items: &[ThreadItem]) -> BTreeMap<String, DrawnSurface> {
    let mut out: BTreeMap<String, DrawnSurface> = BTreeMap::new();
    for item in items {
        if let ThreadItem::Drawn { surface } = item {
            let newer = out
                .get(&surface.surface_id)
                .is_none_or(|known| known.version < surface.version);
            if newer {
                out.insert(surface.surface_id.clone(), (**surface).clone());
            }
        }
    }
    out
}

fn walk<'a>(node: &'a DrawnNode, out: &mut Vec<&'a DrawnNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn nodes(surface: &DrawnSurface) -> Vec<&DrawnNode> {
    let mut out = Vec::new();
    if let Some(root) = &surface.root {
        walk(root, &mut out);
    }
    out
}

#[tokio::test]
async fn a_home_shows_every_sample_drawing_in_adk_uis_catalogs() {
    let home = home().await;
    let items = items(&home).await;
    let refused: Vec<String> = items
        .iter()
        .filter_map(|item| match item {
            ThreadItem::DrawnRefused { refusal } => Some(refusal.reason.clone()),
            _ => None,
        })
        .collect();
    assert!(refused.is_empty(), "refused: {refused:?}");
    let drawn = latest(&items);
    assert_eq!(drawn.len(), 6, "{:?}", drawn.keys().collect::<Vec<_>>());
    assert!(
        drawn
            .values()
            .all(|surface| surface.catalog == "studio 1.0")
    );
    assert_eq!(drawn["throughput"].version, 3, "drawn, then patched twice");

    // The release room is an app whose home view is tabs over its pages.
    let release = &drawn["release-room"];
    let root = release.root.as_ref().unwrap();
    assert_eq!(root.extension.as_ref().unwrap().component, "App");
    let tabs = nodes(release)
        .into_iter()
        .find(|node| node.component == "Tabs")
        .expect("tabs");
    assert_eq!(tabs.tabs, ["Overview", "Checklist", "Deploys"]);

    // The incident's latency chart is a native chart with the server's summary.
    let incident = nodes(&drawn["incident-2041"]);
    let chart = incident
        .iter()
        .filter_map(|node| node.chart.as_ref())
        .find(|chart| chart.kind == ChartKind::Line)
        .expect("a line chart");
    let names: Vec<&str> = chart
        .series
        .iter()
        .map(|series| series.name.as_str())
        .collect();
    assert_eq!(names, ["Checkout", "Search"]);
    assert_eq!(
        chart.summary,
        "Line chart of Checkout and Search, 12 points, from 220 to 840"
    );
    assert!(!chart.ticks.is_empty(), "a time axis has ticks");

    // The warehouse scene's home view names what it holds; its form keeps its field.
    let warehouse = nodes(&drawn["warehouse-bays"]);
    assert!(
        warehouse
            .iter()
            .any(|node| node.text.as_deref() == Some("Dock 3 · free · box"))
    );
    assert!(warehouse.iter().any(|node| {
        node.field
            .as_ref()
            .is_some_and(|field| field.path == "/dock")
    }));

    surfaces_for_the_custom_host(&drawn);
}

/// The projected drawings the custom host shows offline, their times fixed.
fn surfaces_for_the_custom_host(drawn: &BTreeMap<String, DrawnSurface>) {
    let order: Vec<String> = common::fixtures()
        .iter()
        .map(|fixture| fixture.screen["surfaceId"].as_str().unwrap().to_string())
        .collect();
    let surfaces: Vec<Value> = order
        .iter()
        .map(|id| {
            let mut surface = drawn[id].clone();
            let at = surface.at_text.clone();
            for text in [
                &mut surface.drawn_text,
                &mut surface.changed_text,
                &mut surface.earlier_text,
            ] {
                *text = text.replace(&at, "10:02");
            }
            surface.at_text = "10:02".into();
            surface.drawn_at = Utc.with_ymd_and_hms(2026, 11, 16, 10, 2, 0).unwrap();
            surface.call_id = None;
            serde_json::to_value(surface).unwrap()
        })
        .collect();
    let text = format!("{}\n", serde_json::to_string_pretty(&surfaces).unwrap());
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/custom_host/public/surfaces.json");
    if std::env::var_os("ADK_UI_WRITE_SURFACES").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        return;
    }
    let written = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        written,
        text,
        "{} is stale: run ADK_UI_WRITE_SURFACES=1 cargo test --test home_tests",
        path.display()
    );
}

#[tokio::test]
async fn a_form_in_a_drawing_is_held_for_the_person_with_only_its_fields() {
    let home = home().await;
    let mut values = Map::new();
    values.insert("/reason".into(), json!("Latency doubled after the deploy"));
    let outcome = home
        .engine
        .execute(
            &web(),
            Command::SubmitDrawn {
                session_id: SESSION.into(),
                surface_id: "incident-2041".into(),
                version: 1,
                action_id: "rollback".into(),
                values,
            },
        )
        .await
        .expect("the form is sent");
    assert!(
        matches!(outcome, CommandOutcome::FormSent { .. }),
        "{outcome:?}"
    );
    let asked = home.source.asked.lock().unwrap();
    assert_eq!(asked.len(), 1, "one call is asked for, none run");
    assert_eq!(asked[0].tool_name, "start_rollback");
    assert_eq!(
        asked[0].args,
        json!({ "service": "checkout-api", "toVersion": "2026.11.16-1", "reason": "Latency doubled after the deploy" })
    );
}
