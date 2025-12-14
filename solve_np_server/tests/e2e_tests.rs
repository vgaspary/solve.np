use nix::sys::signal;
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use once_cell::sync::Lazy;
use reqwest::cookie::Jar;
use rusqlite::params;
use std::env;
use std::io::Write;
use std::net::TcpListener;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
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
    client: reqwest::Client,
    session_store_path: Option<String>,
}

impl TestApp {
    fn pass(self) {
        Command::new("rm")
            .arg("-f")
            .arg(&self.db_path)
            .output()
            .expect("Failed to remove passed test database");
        if let Some(path) = &self.session_store_path {
            Command::new("rm")
                .arg("-f")
                .arg(path)
                .output()
                .expect("Failed to remove session store file");
        }
    }

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

impl Drop for TestApp {
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

async fn spawn_app(extra_args: &[&str]) -> TestApp {
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

    let mut command = Command::new(&server_binary_path);
    command
        .arg("--db-path")
        .arg(&db_path)
        .arg("--key")
        .arg(random_key)
        .arg("--port")
        .arg(port.to_string());

    command.args(extra_args);

    let session_store_path = extra_args
        .windows(2)
        .find(|w| w[0] == "--session-store")
        .map(|w| w[1].to_string());

    let server_process = command.spawn().expect("Failed to start solve_np_server");

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

    TestApp {
        address,
        db_path,
        server_process,
        client,
        session_store_path,
    }
}

async fn get_solution_id(test: &TestApp) -> String {
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
    caps[1].to_owned()
}

#[tokio::test]
async fn health_check_works() {
    let test = spawn_app(&[]).await;
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
    let test = spawn_app(&[]).await;
    let response = test
        .client
        .get(&format!("{}/", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("Welcome to solveNP.now"));
    test.pass();
}

#[tokio::test]
async fn login_page_get_works() {
    let test = spawn_app(&[]).await;
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
    let test = spawn_app(&[]).await;
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
    let test = spawn_app(&[]).await;
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
    let test = spawn_app(&[]).await;
    let response = test
        .client
        .get(&format!("{}/problems/graph_coloring", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content = response.text().await.unwrap();
    assert!(content.contains("<title>Graph Coloring - solveNP</title>"));
    test.pass();
}

#[tokio::test]
async fn p_vs_np_page_works() {
    let test = spawn_app(&[]).await;
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
    let test = spawn_app(&[]).await;
    test.log_in().await;

    let home_page_response = test
        .client
        .get(&format!("{}/", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(home_page_response.status().is_success());
    let content = home_page_response.text().await.unwrap();
    println!("{content}");
    assert!(content.contains("Welcome to solveNP.now, user!"));
    test.pass();
}

#[tokio::test]
async fn failed_login_works() {
    let test = spawn_app(&[]).await;

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
    let test = spawn_app(&[]).await;
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
    assert!(content.contains("Welcome to solveNP.now!"));
    test.pass();
}

#[tokio::test]
async fn validate_sample_tsp_correct() {
    let test = spawn_app(&[]).await;
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
    let test = spawn_app(&[]).await;
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
    let test = spawn_app(&[]).await;
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
    let test = spawn_app(&[]).await;
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
    let test = spawn_app(&[]).await;
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
    let solution_id = get_solution_id(&test).await;

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

#[tokio::test]
async fn leaderboard_works() {
    let test = spawn_app(&["--rerank-period", "100ms"]).await;
    let mut solution_ids = Vec::new();
    for user_idx in 0..20 {
        let user_name = format!("user_{user_idx}");
        test.log_in_as(&user_name).await;
        test.client
            .post(&format!("{}/api/solutions", &test.address))
            .json(&serde_json::json!({
                "problem_id": "graph_coloring",
                "title": "My best solution.",
                "description": format!("Best solution {user_name} has to offer."),
            }))
            .send()
            .await
            .expect("Failed to create solution");
        let solution_id = get_solution_id(&test).await;
        solution_ids.push(solution_id);
        test.log_out().await;
    }

    let pid = Pid::from_raw(test.server_process.id() as i32);
    signal::kill(pid, Signal::SIGTSTP).expect("Failed to suspend the server.");

    // Mock 'submit_solution' calls by adding DB records in 'submissions' table.
    let mut conn =
        rusqlite::Connection::open(&test.db_path).expect("Failed to connect to the database");
    let tx = conn.transaction().unwrap();
    for user_idx in 0..20 {
        let user_name = format!("user_{user_idx}");
        let solution_id = solution_ids[user_idx].clone();
        let user_id: String = tx
            .query_row(
                "SELECT user_id FROM solutions WHERE solution_id = ?",
                [&solution_id],
                |row| row.get(0),
            )
            .expect("Failed to get user_id from solutions table");

        for input_id in [
            "small_ultra_sparse",
            "small_sparse",
            "small_dense",
            "medium_ultra_sparse",
            "medium_sparse",
            "medium_dense",
            "large_ultra_sparse",
            "large_sparse",
            "large_dense",
        ] {
            let score = 100 + user_idx;
            tx.execute(
                    "INSERT INTO submissions (user_id, user_name, problem_id, solution_id, input_id, solution, hash_digest, status, invalid_message) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        &user_id,
                        &user_name,
                        "graph_coloring",
                        &solution_id,
                        input_id,
                        "",
                        0, // hash_digest
                        0,
                        ""
                    ],
                )
                .expect("Failed to insert submission");
            tx.execute(
                    "UPDATE submissions SET status = ? WHERE user_id = ? AND problem_id = ? and solution_id = ? and input_id = ?;",
                    params![
                        score,
                        &user_id,
                        "graph_coloring",
                        &solution_id,
                        input_id,
                    ],
                )
                .expect("Failed to update submission");
        }
    }
    tx.commit().unwrap();
    signal::kill(pid, Signal::SIGCONT).expect("Failed to resume the server.");

    tokio::time::sleep(Duration::from_millis(200)).await;
    let leaderboard_page_html = test
        .client
        .get(&format!("{}/leaderboard", &test.address))
        .send()
        .await
        .expect("Failed to get leaderboard page")
        .text()
        .await
        .unwrap();
    let re = regex::Regex::new("<td>user_[0-9]+</td>").unwrap();
    let matches = re.find_iter(&leaderboard_page_html).count();
    assert_eq!(matches, 100, "Leaderboard should contain 10 users.");

    test.pass();
}

#[tokio::test]
async fn session_persistence_works() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let session_store_path = format!("{}/test_session_persistence.yaml", manifest_dir);

    // Cleanup before test, just in case
    let _ = std::fs::remove_file(&session_store_path);

    let test1 = spawn_app(&[
        "--session-store",
        &session_store_path,
        "--session-cleanup-period",
        "1s",
    ])
    .await;

    test1.log_in().await;

    // check home page to make sure we are logged in
    let home_page_html = test1
        .client
        .get(format!("{}/", test1.address))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(home_page_html.contains("Welcome to solveNP.now, user!"));

    // Wait for session to be written
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check session file content
    let session_content =
        std::fs::read_to_string(&session_store_path).expect("session file should exist");
    assert!(session_content.contains("user"));

    let client_with_cookie = test1.client.clone();

    // Drop test1 to stop the server
    drop(test1);

    // Spawn a new app instance. It will load from the session file.
    let test2 = spawn_app(&[
        "--session-store",
        &session_store_path,
        "--session-cleanup-period",
        "1s",
    ])
    .await;

    // Use the client from the old app, which has the session cookie
    let home_page_html_2 = client_with_cookie
        .get(format!("{}/", test2.address))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(home_page_html_2.contains("Welcome to solveNP.now, user!"));

    // Now test logout
    client_with_cookie
        .get(format!("{}/api/log_out", test2.address))
        .send()
        .await
        .unwrap();

    // wait for session file to be updated
    tokio::time::sleep(Duration::from_secs(2)).await;

    let session_content_after_logout =
        std::fs::read_to_string(&session_store_path).unwrap_or_default();
    assert!(!session_content_after_logout.contains("user"));

    test2.pass();
}

#[tokio::test]
async fn feedback_works() {
    let test = spawn_app(&[]).await;

    let feedback_form = [
        ("title", "Test Feedback"),
        ("content", "This is a test feedback message."),
        ("email", "test@example.com"),
    ];

    let response = test
        .client
        .post(&format!("{}/api/feedback", &test.address))
        .form(&feedback_form)
        .send()
        .await
        .expect("Failed to submit feedback");

    assert!(response.status().is_success());
    assert_eq!(
        response.url().path(),
        "/feedback",
        "The URL should be the feedback page"
    );
    assert_eq!(
        response.url().query(),
        Some("status=success"),
        "The URL should have the status=success query parameter"
    );

    let content = response.text().await.unwrap();
    assert!(
        content.contains("Feedback sent. Thank you!"),
        "The feedback success message was not found on the page"
    );
    assert!(
        content.contains("<title>Feedback - solveNP</title>"),
        "The title of the feedback page is incorrect"
    );

    test.pass();
}
