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

struct TestApp {
    address: String,
    db_path: String,
    server_process: Child,
    passed: bool,
}

impl Drop for TestApp {
    fn drop(&mut self) {
        println!("Stopping server (PID: {})...", self.server_process.id());
        self.server_process
            .kill()
            .expect("Failed to kill server process");
        self.server_process
            .wait()
            .expect("Failed to wait for server process");
        if self.passed {
            Command::new("rm")
                .arg("-f")
                .arg(&self.db_path)
                .output()
                .expect("Failed to remove passed test database");
        }
        println!("Server stopped.");
    }
}

async fn spawn_app() -> (TestApp, reqwest::Client) {
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
    let static_data_path = format!("{}/static_data.yaml", manifest_dir);

    let server_process = Command::new(&server_binary_path)
        .arg("--static-data-path")
        .arg(&static_data_path)
        .arg("--db-path")
        .arg(&db_path)
        .arg("--key")
        .arg(random_key)
        .arg("--port")
        .arg(port.to_string())
        .arg("--ranking-update-period-secs")
        .arg("3600")
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

    (
        TestApp {
            address,
            db_path,
            server_process,
            passed: false,
        },
        client,
    )
}

#[tokio::test]
async fn health_check_works() {
    let (mut app, client) = spawn_app().await;
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let json: serde_json::Value = response.json().await.unwrap();
    assert!(json["uptime"].is_string());
    app.passed = true;
}

#[tokio::test]
async fn home_page_works() {
    let (mut app, client) = spawn_app().await;
    let response = client
        .get(&format!("{}/", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("Hello"));
    app.passed = true;
}

#[tokio::test]
async fn login_page_get_works() {
    let (mut app, client) = spawn_app().await;
    let response = client
        .get(&format!("{}/log_in", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("Log In"));
    app.passed = true;
}

#[tokio::test]
async fn login_page_get_with_error_works() {
    let (mut app, client) = spawn_app().await;
    let response = client
        .get(&format!(
            "{}/log_in?err=Invalid%20credentials",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("Invalid credentials"));
    app.passed = true;
}

#[tokio::test]
async fn problems_page_works() {
    let (mut app, client) = spawn_app().await;
    let response = client
        .get(&format!("{}/problems", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("<title>Problems"));
    app.passed = true;
}

#[tokio::test]
async fn problem_statement_page_works() {
    let (mut app, client) = spawn_app().await;
    let response = client
        .get(&format!("{}/problems/graph_coloring", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("<title>Graph Coloring - solve.NP</title>"));
    app.passed = true;
}

#[tokio::test]
async fn p_vs_np_page_works() {
    let (mut app, client) = spawn_app().await;
    let response = client
        .get(&format!("{}/p_vs_np", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("P vs NP"));
    app.passed = true;
}

#[tokio::test]
async fn successful_login_works() {
    let (mut app, client) = spawn_app().await;

    let response = client
        .post(&format!("{}/api/log_in", &app.address))
        .form(&[("username", "user"), ("password", "pass")])
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());

    let home_page_response = client
        .get(&format!("{}/", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(home_page_response.status().is_success());
    let content = home_page_response.text().await.unwrap();
    assert!(content.contains("Hello, user!"));
    app.passed = true;
}

#[tokio::test]
async fn failed_login_works() {
    let (mut app, client) = spawn_app().await;

    let response = client
        .post(&format!("{}/api/log_in", &app.address))
        .form(&[("username", "user"), ("password", "user")])
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("Invalid credentials"));
    app.passed = true;
}

#[tokio::test]
async fn logout_works() {
    let (mut app, client) = spawn_app().await;

    // First, log in
    client
        .post(&format!("{}/api/log_in", &app.address))
        .form(&[("username", "user"), ("password", "pass")])
        .send()
        .await
        .expect("Failed to execute request.");

    // Then, log out
    let response = client
        .get(&format!("{}/api/log_out", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());

    // Check home page content after logout
    let home_page_response = client
        .get(&format!("{}/", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(home_page_response.status().is_success());
    let content = home_page_response.text().await.unwrap();
    assert!(content.contains("Hello, Unknown user!"));
    app.passed = true;
}

#[tokio::test]
async fn validate_sample_tsp_correct() {
    let (mut app, client) = spawn_app().await;
    let tsp_solution = serde_json::json!({
        "TspSolution": [0, 1, 4, 2, 6, 5, 3, 7, 0]
    });

    let response = client
        .post(&format!("{}/api/validate_sample", &app.address))
        .json(&tsp_solution)
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json, serde_json::json!({"Valid": 69}));
    app.passed = true;
}

#[tokio::test]
async fn validate_sample_tsp_incorrect() {
    let (mut app, client) = spawn_app().await;
    let tsp_solution = serde_json::json!({
        "TspSolution": [0, 1, 2, 3]
    });

    let response = client
        .post(&format!("{}/api/validate_sample", &app.address))
        .json(&tsp_solution)
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let json: serde_json::Value = response.json().await.unwrap();
    assert!(json["Invalid"].is_string());
    app.passed = true;
}

#[tokio::test]
async fn validate_sample_graph_coloring_incorrect() {
    let (mut app, client) = spawn_app().await;
    let gc_solution = serde_json::json!({
        "GraphColoringSolution": [0, 0, 0, 0, 0, 0, 0, 0]
    });

    let response = client
        .post(&format!("{}/api/validate_sample", &app.address))
        .json(&gc_solution)
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let json: serde_json::Value = response.json().await.unwrap();
    assert!(json["Invalid"].is_string());
    app.passed = true;
}

#[tokio::test]
async fn vote_works() {
    let (mut app, client) = spawn_app().await;

    // Log in as 'user'
    client
        .post(&format!("{}/api/log_in", &app.address))
        .form(&[("username", "user"), ("password", "pass")])
        .send()
        .await
        .expect("Failed to log in as user");

    // First vote for 'user'
    let response = client
        .post(&format!("{}/api/vote/eq", &app.address))
        .send()
        .await
        .expect("Failed to cast first vote");
    assert!(response.status().is_success());

    // Second vote for 'user' should fail
    let response = client
        .post(&format!("{}/api/vote/neq", &app.address))
        .send()
        .await
        .expect("Failed to cast second vote");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    // Log out
    client
        .get(&format!("{}/api/log_out", &app.address))
        .send()
        .await
        .expect("Failed to log out");

    // Log in as 'another_user'
    client
        .post(&format!("{}/api/log_in", &app.address))
        .form(&[("username", "another_user"), ("password", "pass")])
        .send()
        .await
        .expect("Failed to log in as another_user");

    // Vote for 'another_user'
    let response = client
        .post(&format!("{}/api/vote/neq", &app.address))
        .send()
        .await
        .expect("Failed to cast vote for another_user");
    assert!(response.status().is_success());
    app.passed = true;
}

#[tokio::test]
async fn vote_invalid_fails() {
    let (mut app, client) = spawn_app().await;

    // Log in as 'user'
    client
        .post(&format!("{}/api/log_in", &app.address))
        .form(&[("username", "user"), ("password", "pass")])
        .send()
        .await
        .expect("Failed to log in as user");

    // Invalid vote
    let response = client
        .post(&format!("{}/api/vote/invalid", &app.address))
        .send()
        .await
        .expect("Failed to cast invalid vote");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    app.passed = true;
}

#[tokio::test]
async fn solutions_api_works() {
    let (mut app, client) = spawn_app().await;

    // Log in as 'user'
    client
        .post(&format!("{}/api/log_in", &app.address))
        .form(&[("username", "user"), ("password", "pass")])
        .send()
        .await
        .expect("Failed to log in as user");

    // Create a solution
    let create_solution_response = client
        .post(&format!("{}/api/solutions", &app.address))
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
    let solutions_page_html = client
        .get(&format!("{}/solutions", &app.address))
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
    let status_response = client
        .get(&format!(
            "{}/api/solutions/{}/status",
            &app.address, solution_id
        ))
        .send()
        .await
        .expect("Failed to get solution status");
    assert_eq!(status_response.status(), reqwest::StatusCode::OK);
    let status: String = status_response.json().await.unwrap();
    assert_eq!(status, "Untested");

    // Submit a solution
    let submit_response = client
        .post(&format!("{}/api/submit_solution/graph_coloring/{}/small_ultra_sparse", &app.address, solution_id))
        .json(&serde_json::json!({"GraphColoringSolution":[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99]}))
        .send()
        .await
        .expect("Failed to submit solution");
    assert!(submit_response.status().is_success());
    let submit_json: serde_json::Value = submit_response.json().await.unwrap();
    assert!(submit_json["Valid"].is_number());

    // Check submission status
    let submission_status_response = client
        .get(&format!(
            "{}/api/solutions/{}/input/small_ultra_sparse/status",
            &app.address, solution_id
        ))
        .send()
        .await
        .expect("Failed to get submission status");
    assert_eq!(submission_status_response.status(), reqwest::StatusCode::OK);
    let submission_status: serde_json::Value = submission_status_response.json().await.unwrap();
    assert_eq!(submission_status, serde_json::json!({"Solved": 100}));

    // Delete the solution
    let delete_response = client
        .delete(&format!("{}/api/solutions/{}", &app.address, solution_id))
        .send()
        .await
        .expect("Failed to delete solution");
    assert_eq!(delete_response.status(), reqwest::StatusCode::NO_CONTENT);
    app.passed = true;
}
