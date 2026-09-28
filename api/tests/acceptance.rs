//! Acceptance: Stories 1–3 replayed over HTTP against an in-process session.
#![allow(clippy::unwrap_used, clippy::panic)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use plot_twist_api::{ApiHandle, start};
use plot_twist_core::session::Session;
use plot_twist_core::settings::SettingsStore;
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};

struct Api {
    handle: ApiHandle,
    client: Client,
    _dir: tempfile::TempDir,
    discovery: PathBuf,
}

impl Api {
    async fn start() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let discovery = dir.path().join("api.json");
        let session = Arc::new(Session::new(SettingsStore::in_memory()));
        let handle = start(session, 0, discovery.clone()).await.unwrap();
        Self {
            handle,
            client: Client::new(),
            _dir: dir,
            discovery,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.handle.base_url())
    }

    async fn get(&self, path: &str) -> (StatusCode, Value) {
        let response = self.client.get(self.url(path)).send().await.unwrap();
        (response.status(), response.json().await.unwrap())
    }

    async fn send(&self, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
        let request = match method {
            "POST" => self.client.post(self.url(path)),
            _ => self.client.put(self.url(path)),
        };
        let response = request.json(&body).send().await.unwrap();
        (response.status(), response.json().await.unwrap())
    }

    async fn load(&self, path: &Path) -> (StatusCode, Value) {
        self.send("POST", "/load", json!({ "path": path })).await
    }
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/csv")
        .join(name)
        .canonicalize()
        .unwrap()
}

#[tokio::test]
async fn load_plot_and_state_round_trip() {
    let api = Api::start().await;
    assert!(api.handle.base_url().starts_with("http://127.0.0.1:"));
    let discovery: Value =
        serde_json::from_str(&std::fs::read_to_string(&api.discovery).unwrap()).unwrap();
    assert_eq!(discovery["base_url"], api.handle.base_url());

    let (status, body) = api.get("/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["api_version"], 1);

    let (status, loaded) = api.load(&fixture("sensors.csv")).await;
    assert_eq!(status, StatusCode::OK);
    let dataset = &loaded["dataset"];
    assert_eq!(dataset["row_count"], 240);
    assert_eq!(dataset["columns"][0]["name"], "time");
    assert_eq!(dataset["columns"][0]["kind"], "datetime");
    let id = dataset["id"].as_str().unwrap().to_owned();

    let plot = json!({ "dataset_id": id, "x": "time", "y": ["temp", "pressure"], "style": "line" });
    let (status, stored) = api.send("PUT", "/plot", plot.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(stored, plot);
    let (_, state) = api.get("/state").await;
    assert_eq!(state["plot"], plot);
    assert_eq!(state["dataset"]["id"], id);

    let (status, view) = api
        .get(&format!(
            "/datasets/{id}/view?x_min=1790467200&x_max=1790553600&y_min=-100&y_max=2000&width=800&height=400"
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view["series"][0]["column"], "temp");
    assert!(!view["series"][0]["points"].as_array().unwrap().is_empty());

    let mut handle = api.handle;
    handle.stop();
    assert!(!api.discovery.exists(), "discovery file removed on stop");
}

#[tokio::test]
async fn plot_errors_leave_the_plot_unchanged() {
    let api = Api::start().await;
    let (_, first) = api.load(&fixture("sensors.csv")).await;
    let stale = first["dataset"]["id"].as_str().unwrap().to_owned();
    let (_, loaded) = api.load(&fixture("sensors.csv")).await;
    let id = loaded["dataset"]["id"].as_str().unwrap().to_owned();
    let good = json!({ "dataset_id": id, "x": "time", "y": ["temp"], "style": "scatter" });
    api.send("PUT", "/plot", good.clone()).await;

    let cases = [
        (
            json!({ "dataset_id": id, "x": "time", "y": ["nope"], "style": "line" }),
            404,
            "UNKNOWN_COLUMN",
        ),
        (
            json!({ "dataset_id": id, "x": "time", "y": ["site"], "style": "line" }),
            422,
            "COLUMN_NOT_USABLE_AS_Y",
        ),
        (
            json!({ "dataset_id": id, "x": "temp", "y": ["temp"], "style": "line" }),
            422,
            "X_ALSO_Y",
        ),
        (
            json!({ "dataset_id": stale, "x": "time", "y": ["temp"], "style": "line" }),
            409,
            "STALE_DATASET",
        ),
        (
            json!({ "dataset_id": "ds-999", "x": "time", "y": ["temp"], "style": "line" }),
            404,
            "UNKNOWN_DATASET",
        ),
        (json!({ "x": "time" }), 400, "BAD_REQUEST"),
    ];
    for (body, status, code) in cases {
        let (got, report) = api.send("PUT", "/plot", body).await;
        assert_eq!(got.as_u16(), status, "{code}");
        assert_eq!(report["code"], code);
        assert!(!report["hint"].as_str().unwrap().is_empty());
    }
    let (_, state) = api.get("/state").await;
    assert_eq!(state["plot"], good);
}

#[tokio::test]
async fn malformed_file_reports_line_and_keeps_previous_dataset() {
    let api = Api::start().await;
    let (_, loaded) = api.load(&fixture("sensors.csv")).await;
    let (status, report) = api.load(&fixture("ragged.csv")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(report["code"], "FIELD_COUNT_MISMATCH");
    assert_eq!(report["line"], 3);
    assert_eq!(report["details"]["expected_fields"], 3);
    assert_eq!(report["details"]["actual_fields"], 2);
    let (_, state) = api.get("/state").await;
    assert_eq!(state["dataset"]["id"], loaded["dataset"]["id"]);
}

#[tokio::test]
async fn bad_cells_load_with_preview_and_paging() {
    let api = Api::start().await;
    let (status, loaded) = api.load(&fixture("bad_cells.csv")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(loaded["dataset"]["bad_cell_total"], 3);
    assert_eq!(loaded["bad_cells_preview"][2]["line"], 1204);
    assert_eq!(loaded["bad_cells_preview"][2]["text"], "abc");
    let id = loaded["dataset"]["id"].as_str().unwrap();
    let (status, page) = api
        .get(&format!("/datasets/{id}/bad-cells?offset=1&limit=1"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page["total"], 3);
    assert_eq!(page["items"][0]["line"], 60);
    let (status, report) = api.get("/datasets/ds-999/bad-cells").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(report["code"], "UNKNOWN_DATASET");
}

#[tokio::test]
async fn second_load_while_loading_is_refused() {
    let api = Api::start().await;
    let dir = tempfile::tempdir().unwrap();
    let big = dir.path().join("big.csv");
    let mut file = std::io::BufWriter::new(std::fs::File::create(&big).unwrap());
    writeln!(file, "x,a,b,c").unwrap();
    for i in 0..600_000 {
        writeln!(file, "{i},{}.5,{}.25,{}", i % 97, i % 13, i % 7).unwrap();
    }
    file.flush().unwrap();
    drop(file);

    let first = {
        let client = api.client.clone();
        let url = api.url("/load");
        let body = json!({ "path": big });
        tokio::spawn(async move { client.post(url).json(&body).send().await.unwrap().status() })
    };
    let mut refused = false;
    for _ in 0..200 {
        let (_, state) = api.get("/state").await;
        if !state["loading"].is_null() {
            let (status, report) = api.load(&fixture("sensors.csv")).await;
            assert_eq!(status, StatusCode::CONFLICT);
            assert_eq!(report["code"], "LOAD_IN_PROGRESS");
            refused = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    assert!(
        refused,
        "the big load finished before a second load could be attempted"
    );
    assert_eq!(first.await.unwrap(), StatusCode::OK);
}

#[tokio::test]
async fn requests_from_other_hosts_are_rejected() {
    let api = Api::start().await;
    let response = api
        .client
        .get(api.url("/state"))
        .header("Host", "evil.example:80")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
