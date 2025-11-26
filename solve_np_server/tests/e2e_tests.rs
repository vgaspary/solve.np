use once_cell::sync::Lazy;
use reqwest::cookie::Jar;
use std::env;
use std::io::Write;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

static E2E_TEST_BUILD: Lazy<()> = Lazy::new(|| {
    println!("Building solve_np_server for E2E tests...");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output = Command::new("cargo")
        .arg("build")
        .current_dir(manifest_dir)
        .output()
        .expect("Failed to build solve_np_server");
    if !output.status.success() {
        panic!(
            "Failed to build solve_np_server: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    println!("Build successful.");
});

struct Test {
    address: String,
    db_path: String,
    server_process: Child,
    client: reqwest::Client,
}

impl Test {
    fn pass(self) {
        Command::new("rm")
            .arg("-f")
            .arg(&self.db_path)
            .output()
            .expect("Failed to remove passed test database");
    }
}

impl Drop for Test {
    fn drop(&mut self) {
        println!("Stopping server (PID: {})...", self.server_process.id());
        self.server_process
            .kill()
            .expect("Failed to kill server process");
        self.server_process
            .wait()
            .expect("Failed to wait for server process");
        println!("Server stopped.");
    }
}

async fn spawn_app() -> Test {
    Lazy::force(&E2E_TEST_BUILD);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let db_path = format!("{}/test_file_{}.db3", manifest_dir, port);
    let init_sql_path = format!("{}/init.sql", manifest_dir);

    Command::new("rm")
        .arg("-f")
        .arg(&db_path)
        .output()
        .expect("Failed to remove old test database");

    let init_sql_content = std::fs::read_to_string(init_sql_path).expect("Failed to read init.sql");

    let mut cmd = Command::new("sqlite3");
    cmd.arg(&db_path);
    cmd.stdin(Stdio::piped());
    let mut child = cmd.spawn().expect("Failed to spawn sqlite3 process");
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(init_sql_content.as_bytes())
        .expect("Failed to write to sqlite3 stdin");
    drop(stdin);
    child
        .wait()
        .expect("sqlite3 process did not exit successfully");

    let random_key = "YWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYQ==";
    let server_binary_path = format!("{}/../target/debug/solve_np_server", manifest_dir);

    let server_process = Command::new(&server_binary_path)
        .arg("--db-path")
        .arg(&db_path)
        .arg("--key")
        .arg(random_key)
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start solve_np_server");

    let address = format!("http://127.0.0.1:{}", port);

    let client = reqwest::Client::new();
    for _ in 0..20 {
        if client
            .get(&format!("{}/health_check", address))
            .send()
            .await
            .is_ok()
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let cookie_jar = Arc::new(Jar::default());
    let client = reqwest::Client::builder()
        .cookie_provider(cookie_jar)
        .build()
        .unwrap();

    Test {
        address,
        db_path,
        server_process,
        client,
    }
}

impl Test {
    async fn log_in(&self) {
        self.log_in_as("user").await;
    }

    async fn log_in_as(&self, username: &str) {
        self.client
            .post(&format!("{}/api/log_in", &self.address))
            .form(&[("username", username), ("password", "pass")])
            .send()
            .await
            .expect("Failed to log in as user");
    }

    async fn log_out(&self) {
        self.client
            .get(&format!("{}/api/log_out", &self.address))
            .send()
            .await
            .expect("Failed to log out");
    }
}

#[tokio::test]
async fn health_check_works() {
    let test = spawn_app().await;
    let response = test
        .client
        .get(&format!("{}/health_check", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let json: serde_json::Value = response.json().await.unwrap();
    assert!(json["uptime"].is_string());
    test.pass();
}

#[tokio::test]
async fn home_page_works() {
    let test = spawn_app().await;
    let response = test
        .client
        .get(&format!("{}/", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("Hello"));
    test.pass();
}

#[tokio::test]
async fn login_page_get_works() {
    let test = spawn_app().await;
    let response = test
        .client
        .get(&format!("{}/log_in", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("Log In"));
    test.pass();
}

#[tokio::test]
async fn login_page_get_with_error_works() {
    let test = spawn_app().await;
    let response = test
        .client
        .get(&format!(
            "{}/log_in?err=Invalid%20credentials",
            &test.address
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("Invalid credentials"));
    test.pass();
}

#[tokio::test]
async fn problems_page_works() {
    let test = spawn_app().await;
    let response = test
        .client
        .get(&format!("{}/problems", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("<title>Problems"));
    test.pass();
}

#[tokio::test]
async fn problem_statement_page_works() {
    let test = spawn_app().await;
    let response = test
        .client
        .get(&format!("{}/problems/graph_coloring", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("<title>Graph Coloring - solve.NP</title>"));
    test.pass();
}

#[tokio::test]
async fn p_vs_np_page_works() {
    let test = spawn_app().await;
    let response = test
        .client
        .get(&format!("{}/p_vs_np", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("P vs NP"));
    test.pass();
}

#[tokio::test]
async fn successful_login_works() {
    let test = spawn_app().await;
    test.log_in().await;

    let home_page_response = test
        .client
        .get(&format!("{}/", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(home_page_response.status().is_success());
    let content = home_page_response.text().await.unwrap();
    assert!(content.contains("Hello, user!"));
    test.pass();
}

#[tokio::test]
async fn failed_login_works() {
    let test = spawn_app().await;

    let response = test
        .client
        .post(&format!("{}/api/log_in", &test.address))
        .form(&[("username", "user"), ("password", "user")])
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("Invalid credentials"));
    test.pass();
}

#[tokio::test]
async fn logout_works() {
    let test = spawn_app().await;
    test.log_in().await;

    let response = test
        .client
        .get(&format!("{}/api/log_out", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());

    // Check home page content after logout
    let home_page_response = test
        .client
        .get(&format!("{}/", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(home_page_response.status().is_success());
    let content = home_page_response.text().await.unwrap();
    assert!(content.contains("Hello, Unknown user!"));
    test.pass();
}

#[tokio::test]
async fn validate_sample_tsp_correct() {
    let test = spawn_app().await;
    let tsp_solution = serde_json::json!({
        "TspSolution": [0, 1, 4, 2, 6, 5, 3, 7, 0]
    });

    let response = test
        .client
        .post(&format!("{}/api/validate_sample", &test.address))
        .json(&tsp_solution)
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json, serde_json::json!({"Valid": 69}));
    test.pass();
}

#[tokio::test]
async fn validate_sample_tsp_incorrect() {
    let test = spawn_app().await;
    let tsp_solution = serde_json::json!({
        "TspSolution": [0, 1, 2, 3]
    });

    let response = test
        .client
        .post(&format!("{}/api/validate_sample", &test.address))
        .json(&tsp_solution)
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let json: serde_json::Value = response.json().await.unwrap();
    assert!(json["Invalid"].is_string());
    test.pass();
}

#[tokio::test]
async fn validate_sample_graph_coloring_incorrect() {
    let test = spawn_app().await;
    let gc_solution = serde_json::json!({
        "GraphColoringSolution": [0, 0, 0, 0, 0, 0, 0, 0]
    });

    let response = test
        .client
        .post(&format!("{}/api/validate_sample", &test.address))
        .json(&gc_solution)
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let json: serde_json::Value = response.json().await.unwrap();
    assert!(json["Invalid"].is_string());
    test.pass();
}

#[tokio::test]
async fn vote_works() {
    let test = spawn_app().await;
    test.log_in_as("user").await;

    // First vote for 'user'
    let response = test
        .client
        .post(&format!("{}/api/vote/eq", &test.address))
        .send()
        .await
        .expect("Failed to cast first vote");
    assert!(response.status().is_success());

    // Second vote for 'user' should fail
    let response = test
        .client
        .post(&format!("{}/api/vote/neq", &test.address))
        .send()
        .await
        .expect("Failed to cast second vote");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    test.log_out().await;
    test.log_in_as("another_user").await;

    // Vote for 'another_user'
    let response = test
        .client
        .post(&format!("{}/api/vote/neq", &test.address))
        .send()
        .await
        .expect("Failed to cast vote for another_user");
    assert!(response.status().is_success());

    // Invalid vote
    let response = test
        .client
        .post(&format!("{}/api/vote/invalid", &test.address))
        .send()
        .await
        .expect("Failed to cast invalid vote");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    test.pass();
}

#[tokio::test]
async fn solutions_api_works() {
    let test = spawn_app().await;
    test.log_in().await;

    // Create a solution
    let create_solution_response = test
        .client
        .post(&format!("{}/api/solutions", &test.address))
        .json(&serde_json::json!({
            "problem_id": "graph_coloring",
            "title": "My test solution",
            "description": "A description"
        }))
        .send()
        .await
        .expect("Failed to create solution");
    assert!(create_solution_response.status().is_success());

    // Get the solution ID from the solutions page
    let solutions_page_html = test
        .client
        .get(&format!("{}/solutions", &test.address))
        .send()
        .await
        .expect("Failed to get solutions page")
        .text()
        .await
        .unwrap();

    let re = regex::Regex::new(r"'([a-f0-9]{32})'").unwrap();
    let caps = re
        .captures(&solutions_page_html)
        .expect("Could not find solution ID with regex");
    let solution_id = &caps[1];

    // Check solution status
    let status_response = test
        .client
        .get(&format!(
            "{}/api/solutions/{}/status",
            &test.address, solution_id
        ))
        .send()
        .await
        .expect("Failed to get solution status");
    assert_eq!(status_response.status(), reqwest::StatusCode::OK);
    let status: String = status_response.json().await.unwrap();
    assert_eq!(status, "Untested");

    // Submit a solution
    let submit_response = test
        .client
        .post(&format!(
            "{}/api/submit_solution/graph_coloring/{}/small_ultra_sparse",
            &test.address, solution_id
        ))
        .json(&serde_json::json!({"GraphColoringSolution": (0..100).collect::<Vec<_>>()}))
        .send()
        .await
        .expect("Failed to submit solution");
    assert!(submit_response.status().is_success());
    let submit_json: serde_json::Value = submit_response.json().await.unwrap();
    assert!(submit_json["Valid"].is_number());

    // Check submission status
    let submission_status_response = test
        .client
        .get(&format!(
            "{}/api/solutions/{}/input/small_ultra_sparse/status",
            &test.address, solution_id
        ))
        .send()
        .await
        .expect("Failed to get submission status");
    assert_eq!(submission_status_response.status(), reqwest::StatusCode::OK);
    let submission_status: serde_json::Value = submission_status_response.json().await.unwrap();
    assert_eq!(submission_status, serde_json::json!({"Solved": 100}));

    // Delete the solution
    let delete_response = test
        .client
        .delete(&format!("{}/api/solutions/{}", &test.address, solution_id))
        .send()
        .await
        .expect("Failed to delete solution");
    assert_eq!(delete_response.status(), reqwest::StatusCode::NO_CONTENT);
    test.pass();
}
