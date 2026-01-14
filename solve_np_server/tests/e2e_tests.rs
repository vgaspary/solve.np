use futures::future::join_all;
use nix::sys::signal;
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use once_cell::sync::Lazy;
use reqwest::cookie::Jar;
use rusqlite::params;
use serde_json::json;
use std::collections::HashMap;
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

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct StaticData {
    problems: Vec<Problem>,
}

#[derive(Debug, Clone, Deserialize)]
struct Problem {
    id: String,
    name: String,
    inputs: Vec<Input>,
}

#[derive(Debug, Clone, Deserialize)]
struct Input {
    id: String,
    num_vertices: usize,
}

#[derive(Debug, Clone, Copy)]
enum SortingOrder {
    Minimize,
    Maximize,
}

fn get_problems() -> Vec<Problem> {
    let static_data_content =
        std::fs::read_to_string("static_data.yaml").expect("Failed to read static_data.yaml");
    let static_data: StaticData =
        serde_yaml::from_str(&static_data_content).expect("Failed to parse static_data.yaml");
    static_data.problems
}

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

    fn pass_and_preserve_sessions(self) {
        Command::new("rm")
            .arg("-f")
            .arg(&self.db_path)
            .output()
            .expect("Failed to remove passed test database");
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
        let pid = Pid::from_raw(self.server_process.id() as i32);
        signal::kill(pid, Signal::SIGTERM).ok();
        match self.server_process.wait() {
            Ok(status) => println!("Server process exited with status: {}", status),
            Err(e) => eprintln!(
                "Failed to wait for server process: {}. It might have been already reaped.",
                e
            ),
        }
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
async fn all_problem_pages_work() {
    let test = spawn_app(&[]).await;
    let problems = get_problems();
    let futures = problems.into_iter().map(|problem| {
        let client = test.client.clone();
        let address = test.address.clone();
        async move {
            let response = client
                .get(&format!("{}/problems/{}", &address, problem.id))
                .send()
                .await
                .expect("Failed to execute request.");

            assert!(response.status().is_success());
            let content = response.text().await.unwrap();
            assert!(content.contains(&format!("<title>{} - solveNP</title>", problem.name)));
        }
    });
    join_all(futures).await;
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
async fn validate_sample_works() {
    let test = spawn_app(&[]).await;
    let problems = get_problems();
    let solutions = HashMap::from([
        (
            "travelling_salesman",
            (
                json!({ "TspSolution": [0, 1, 4, 2, 6, 5, 3, 7, 0] }),
                json!({"Valid": 69}),
            ),
        ),
        (
            "graph_coloring",
            (
                json!({"GraphColoringSolution": [0, 1, 2, 3, 4, 5, 6, 7]}),
                json!({"Valid": 8}),
            ),
        ),
        (
            "vertex_cover",
            (
                json!({"VertexCoverSolution": [0, 1, 2, 3, 4, 5, 6, 7]}),
                json!({"Valid": 8}),
            ),
        ),
        (
            "maximum_clique",
            (
                json!({"MaximumCliqueSolution": [0, 2, 5]}),
                json!({"Valid": 3}),
            ),
        ),
        (
            "maximum_cut",
            (
                json!({"MaximumCutSolution": [0, 2, 5]}),
                json!({"Valid": 47}),
            ),
        ),
        (
            "steiner_tree",
            (
                json!({"SteinerTreeSolution": [(0, 2), (2, 7), (2, 5), (3, 7)]}),
                json!({"Valid": 34}),
            ),
        ),
    ]);
    let client = test.client.clone();
    let address = test.address.clone();
    let futures = problems.into_iter().map(|problem| {
        let (solution, result) = solutions.get(&problem.id.as_str()).cloned().unwrap();
        let client = client.clone();
        let address = address.clone();
        async move {
            let response = client
                .post(&format!("{}/api/validate_sample", address))
                .json(&solution)
                .send()
                .await
                .expect("Failed to execute request.");

            assert!(response.status().is_success());
            let json: serde_json::Value = response.json().await.unwrap();
            assert_eq!(json, result);
        }
    });
    join_all(futures).await;
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

async fn solutions_api_works_for_problem(
    problem_id: String,
    input_id: String,
    solution_json: serde_json::Value,
) {
    let test = spawn_app(&[]).await;
    test.log_in().await;

    // Create a solution
    let create_solution_response = test
        .client
        .post(&format!("{}/api/solutions", &test.address))
        .json(&json!({
            "problem_id": problem_id,
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
            "{}/api/submit_solution/{}/{}/{}",
            &test.address, problem_id, solution_id, input_id
        ))
        .json(&solution_json)
        .send()
        .await
        .expect("Failed to submit solution");
    assert!(submit_response.status().is_success());
    let submit_json: serde_json::Value = submit_response.json().await.unwrap();
    assert!(
        submit_json["Valid"].is_number() || submit_json["Invalid"].is_string(),
        "Expected either Valid score or Invalid message, got: {:?}",
        submit_json
    );
    let score_or_status = if submit_json["Valid"].is_number() {
        json!({"Solved": submit_json["Valid"].as_i64().unwrap()})
    } else {
        json!("Attempted")
    };

    // Check submission status
    let submission_status_response = test
        .client
        .get(&format!(
            "{}/api/solutions/{}/input/{}/status",
            &test.address, solution_id, input_id
        ))
        .send()
        .await
        .expect("Failed to get submission status");
    assert_eq!(submission_status_response.status(), reqwest::StatusCode::OK);
    let submission_status: serde_json::Value = submission_status_response.json().await.unwrap();
    assert_eq!(submission_status, score_or_status);

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
async fn all_solutions_api_work() {
    let problems = get_problems();
    let problem = problems
        .into_iter()
        .find(|p| p.id == "travelling_salesman")
        .unwrap();
    let input = problem.inputs[0].clone();
    let solution = {
        let mut path: Vec<usize> = (0..input.num_vertices).collect();
        path.push(0);
        json!({"TspSolution": path})
    };
    solutions_api_works_for_problem(problem.id, input.id, solution).await;
}

#[tokio::test]
async fn update_solution_works() {
    let test = spawn_app(&[]).await;
    test.log_in().await;

    // Create a solution
    let create_solution_response = test
        .client
        .post(&format!("{}/api/solutions", &test.address))
        .json(&serde_json::json!({
            "problem_id": "graph_coloring",
            "title": "Original title",
            "description": "Original description"
        }))
        .send()
        .await
        .expect("Failed to create solution");
    assert!(create_solution_response.status().is_success());

    // Get the solution ID from the solutions page
    let solution_id = get_solution_id(&test).await;

    // Update the solution
    let updated_title = "Updated title";
    let updated_description = "Updated description";
    let update_solution_response = test
        .client
        .put(&format!("{}/api/solutions/{}", &test.address, solution_id))
        .json(&serde_json::json!({
            "title": updated_title,
            "description": updated_description
        }))
        .send()
        .await
        .expect("Failed to update solution");
    assert!(update_solution_response.status().is_success());

    // Verify the update by fetching the solution page and checking its content
    let solution_page_html = test
        .client
        .get(&format!("{}/solutions/{}", &test.address, solution_id))
        .send()
        .await
        .expect("Failed to get solution page")
        .text()
        .await
        .unwrap();

    assert!(solution_page_html.contains(updated_title));
    assert!(solution_page_html.contains(updated_description));

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

#[ignore]
#[tokio::test]
async fn all_leaderboards_work() {
    let problems = get_problems();
    let test = spawn_app(&["--rerank-period", "1s"]).await;

    let sorting_orders = HashMap::from([
        ("graph_coloring", SortingOrder::Minimize),
        ("travelling_salesman", SortingOrder::Minimize),
        ("vertex_cover", SortingOrder::Minimize),
        ("maximum_clique", SortingOrder::Maximize),
        ("maximum_cut", SortingOrder::Maximize),
    ]);

    for problem in problems {
        let sorting_order = sorting_orders.get(problem.id.as_str()).unwrap();
        for user_idx in 0..20 {
            let user_name = format!("user_{}", user_idx);
            test.log_in_as(&user_name).await;

            let create_solution_response = test
                .client
                .post(&format!("{}/api/solutions", &test.address))
                .json(&json!({
                    "problem_id": problem.id,
                    "title": "My best solution.",
                    "description": format!("Best solution {} has to offer.", user_name),
                }))
                .send()
                .await
                .expect("Failed to create solution");
            assert!(create_solution_response.status().is_success());
            let solution_id = get_solution_id(&test).await;

            // Mock 'submit_solution' calls by adding DB records in 'submissions' table.
            let pid = Pid::from_raw(test.server_process.id() as i32);
            signal::kill(pid, Signal::SIGTSTP).expect("Failed to suspend the server.");
            let mut conn = rusqlite::Connection::open(&test.db_path)
                .expect("Failed to connect to the database");
            let tx = conn.transaction().unwrap();
            let user_name = format!("user_{user_idx}");
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
                    "INSERT INTO submissions (user_id, user_name, problem_id, solution_id, input_id, solution, hash_digest, status, invalid_me
sage) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
            tx.commit().unwrap();
            signal::kill(pid, Signal::SIGCONT).expect("Failed to resume the server.");

            test.log_out().await;
        }

        tokio::time::sleep(Duration::from_millis(1200)).await;
        let leaderboard_page_html = test
            .client
            .get(&format!("{}/leaderboard", &test.address))
            .send()
            .await
            .expect("Failed to get leaderboard page")
            .text()
            .await
            .unwrap();

        let re = regex::Regex::new("<td>user_([0-9]+)</td>").unwrap();
        let matches: Vec<usize> = re
            .captures_iter(&leaderboard_page_html)
            .map(|cap| cap[1].parse().unwrap())
            .collect();

        assert_eq!(
            matches.len(),
            10,
            "Leaderboard should contain 10 users for problem {}",
            problem.id
        );

        let mut sorted_matches = matches.clone();
        sorted_matches.sort();

        let expected_users: Vec<usize> = match sorting_order {
            SortingOrder::Minimize => (0..10).collect(),
            SortingOrder::Maximize => (10..20).rev().collect(),
        };

        assert_eq!(
            matches, expected_users,
            "Users are not sorted correctly on the leaderboard for problem {}",
            problem.id
        );
    }
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

    // Pass test1 to stop the server and clean up its resources
    test1.pass_and_preserve_sessions();

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
