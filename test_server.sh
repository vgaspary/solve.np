#!/bin/bash

set -euo pipefail

ROOT_DIR=$(pwd)
SERVER_DIR="${ROOT_DIR}/solve_np_server"
STATIC_DATA_PATH="${SERVER_DIR}/static_data.yaml"
DB_PATH="${SERVER_DIR}/test_file.db3"
SERVER_ADDRESS="127.0.0.1"
SERVER_PORT=2345
SERVER_URL="http://${SERVER_ADDRESS}:${SERVER_PORT}"

# Create the database and the votes table
rm -f "${DB_PATH}"

sqlite3 "${DB_PATH}" \
  "CREATE TABLE votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT UNIQUE,
    vote BOOLEAN
    );"

sqlite3 "${DB_PATH}" \
      "CREATE TABLE solutions (
        solution_id TEXT PRIMARY KEY,
        user_id TEXT,
        problem_id TEXT,
        title TEXT,
        description TEXT,
        status INTEGER
        );"

sqlite3 "${DB_PATH}" "CREATE UNIQUE INDEX idx_user_problem_title ON solutions (user_id, problem_id, title);"

sqlite3 "${DB_PATH}" \
  "CREATE TABLE submissions (
    user_id TEXT,
    problem_id TEXT,
    solution_id TEXT,
    input_id TEXT,
    solution TEXT,
    hash_digest INTEGER,
    status INTEGER,
    invalid_message TEXT,
    PRIMARY KEY (user_id, problem_id, solution_id, input_id)
    );"

sqlite3 "${DB_PATH}" \
  "CREATE TRIGGER update_solution_status_on_new_submission
AFTER UPDATE OF solution_id, status ON submissions
FOR EACH ROW
  WHEN OLD.status <> NEW.status
  BEGIN
  UPDATE solutions
  SET status = (
    SELECT
      CASE
        WHEN MIN(T1.status) = -2 AND MAX(T1.status) = -2 THEN -2
        WHEN MIN(T1.status) >= 0 THEN 0
      ELSE -1
      END
      FROM submissions AS T1
      WHERE T1.solution_id = NEW.solution_id
    )
    WHERE solution_id = NEW.solution_id;
  END;"

# Generate a random base64 encoded key for the session middleware
RANDOM_KEY=$(head /dev/urandom | tr -dc A-Za-z0-9_ | head -c 64 | base64 -w 0)

echo "Building solve_np_server..."
cd "${SERVER_DIR}" && cargo build

echo "Starting solve_np_server in the background..."
# Start the server and redirect output to a log file
"${ROOT_DIR}/target/debug/solve_np_server" \
  --static-data-path "${STATIC_DATA_PATH}" \
  --db-path "${DB_PATH}" \
  --key "${RANDOM_KEY}" \
  --port ${SERVER_PORT} > test.log 2>&1 &

SERVER_PID=$!
echo "Server started with PID: ${SERVER_PID}"

# Function to clean up server process on exit
cleanup() {
  echo "Stopping server (PID: ${SERVER_PID})..."
  kill -9 "${SERVER_PID}" || true # Use || true to prevent script from exiting if kill fails
  wait "${SERVER_PID}" 2>/dev/null || true # Wait for the process to terminate
  echo "Server stopped."
}

trap cleanup EXIT

echo "Waiting for server to become responsive..."
for i in $(seq 1 10); do
  if curl -s "${SERVER_URL}/health_check" > /dev/null; then
    echo "Server is up!"
    break
  fi
  echo "Attempt ${i}: Server not yet responsive, waiting 1 second..."
  sleep 1
  if [ "$i" -eq 10 ]; then
    echo "Server did not start in time. Exiting."
    exit 1
  fi
done

echo "Running tests..."

# Test 1: Health Check
echo "Testing /health_check..."
HEALTH_CHECK_RESPONSE=$(curl -s "${SERVER_URL}/health_check")
if echo "${HEALTH_CHECK_RESPONSE}" | jq -e '.uptime' > /dev/null; then
  echo -e "/healt_check \e[32mpassed\e[0m."
else
  echo -e "/health_check \e[31mfailed\e[0m. Response: ${HEALTH_CHECK_RESPONSE}"
  exit 1
fi

# Test 4: Home page
echo "Testing / (home page)..."
HOME_PAGE_RESPONSE=$(curl -s "${SERVER_URL}/")
if echo "${HOME_PAGE_RESPONSE}" | grep -q "Hello"; then
  echo -e "/ (home page) \e[32mpassed\e[0m."
else
  echo -e "/ (home page) \e[31mfailed\e[0m. Response snippet: ${HOME_PAGE_RESPONSE:0:200}... (Full response length: ${#HOME_PAGE_RESPONSE}) "
  exit 1
fi

# Test 5: Login Page (GET)
echo "Testing /log_in (GET)..."
LOGIN_PAGE_RESPONSE=$(curl -s "${SERVER_URL}/log_in")
if echo "${LOGIN_PAGE_RESPONSE}" | grep -q "Log In"; then
  echo -e "/log_in (GET) \e[32mpassed\e[0m."
else
  echo -e "/log_in (GET) \e[31mfailed\e[0m. Response snippet: ${LOGIN_PAGE_RESPONSE:0:200}... (Full response length: ${#LOGIN_PAGE_RESPONSE}) "
  exit 1
fi

# Test 6: Login Page (GET) with error
echo "Testing /log_in?err=Invalid%20credentials (GET)..."
LOGIN_PAGE_ERROR_RESPONSE=$(curl -s "${SERVER_URL}/log_in?err=Invalid%20credentials")
if echo "${LOGIN_PAGE_ERROR_RESPONSE}" | grep -q "Invalid credentials"; then
  echo -e "/log_in?err=Invalid%20credentials (GET) \e[32mpassed\e[0m."
else
  echo -e "/log_in?err=Invalid%20credentials (GET) \e[31mfailed\e[0m. Response snippet: ${LOGIN_PAGE_ERROR_RESPONSE:0:200}... (Full response length: ${#LOGIN_PAGE_ERROR_RESPONSE}) "
  exit 1
fi

# Test 7: Successful Login (POST)
echo "Testing /log_in (POST) - successful login..."
# Use a temporary file to store cookies
COOKIE_JAR=$(mktemp)

# Perform the login POST request and capture headers
LOGIN_POST_RESPONSE=$(curl -v -c "${COOKIE_JAR}" -X POST -d "username=user&password=pass" "${SERVER_URL}/api/log_in" 2>&1)

# Check if the login POST resulted in a 303 See Other redirect
if echo "${LOGIN_POST_RESPONSE}" | grep -q "< HTTP/1.1 303 See Other"; then
  echo -e "Login POST redirect status 303 \e[32mreceived\e[0m."
else
  echo -e "Login POST \e[31mfailed\e[0m to redirect with 303. Response: ${LOGIN_POST_RESPONSE}"
  rm "${COOKIE_JAR}"
  exit 1
fi

# Verify that a session cookie was set
if [ -s "${COOKIE_JAR}" ]; then
  echo -e "Session cookie \e[32mfound\e[0m in cookie jar."
else
  echo -e "Session cookie \e[31mnot found\e[0m in cookie jar."
  rm "${COOKIE_JAR}"
  exit 1
fi

# Fetch the home page using the session cookie
HOME_PAGE_AFTER_LOGIN=$(curl -s -b "${COOKIE_JAR}" "${SERVER_URL}/")

if echo "${HOME_PAGE_AFTER_LOGIN}" | grep -q "Hello, user!"; then
  echo -e "Successful login (POST) test \e[32mpassed\e[0m."
else
  echo -e "Successful login (POST) test \e[31mfailed\e[0m. Home page content: ${HOME_PAGE_AFTER_LOGIN:0:200}... (Full response length: ${#HOME_PAGE_AFTER_LOGIN}) "
  rm "${COOKIE_JAR}"
  exit 1
fi
rm "${COOKIE_JAR}"

# Test 8: Failed Login (POST)
echo "Testing /log_in (POST) - failed login..."
# Use a temporary file to store cookies for this test
COOKIE_JAR_FAIL=$(mktemp)

# Perform the failed login POST request and capture headers
LOGIN_FAIL_POST_RESPONSE_HEADERS=$(curl -s -D - -c "${COOKIE_JAR_FAIL}" -X POST -d "username=user&password=user" "${SERVER_URL}/api/log_in" -o /dev/null)

# Check if the failed login POST resulted in a 303 See Other redirect
if echo "${LOGIN_FAIL_POST_RESPONSE_HEADERS}" | grep -q "HTTP/1.1 303 See Other"; then
  echo -e "Failed login POST redirect status 303 \e[32mreceived\e[0m."
else
  echo -e "Failed login POST \e[31mfailed\e[0m to redirect with 303. Headers: ${LOGIN_FAIL_POST_RESPONSE_HEADERS}"
  rm "${COOKIE_JAR_FAIL}"
  exit 1
fi

# Extract the Location header to get the redirect URL
REDIRECT_URL=$(echo "${LOGIN_FAIL_POST_RESPONSE_HEADERS}" | grep -i Location | awk '{print $2}' | tr -d '\r')

# Fetch the redirected login page using the session cookie (if any)
LOGIN_PAGE_AFTER_FAIL=$(curl -s -b "${COOKIE_JAR_FAIL}" "${SERVER_URL}${REDIRECT_URL}")

if echo "${LOGIN_PAGE_AFTER_FAIL}" | grep -q "<p style=\"color:red\">Invalid credentials</p>"; then
  echo -e "Failed login (POST) test \e[32mpassed\e[0m."
else
  echo -e "Failed login (POST) test \e[31mfailed\e[0m. Response snippet: ${LOGIN_PAGE_AFTER_FAIL:0:200}... (Full response length: ${#LOGIN_PAGE_AFTER_FAIL}) "
  rm "${COOKIE_JAR_FAIL}"
  exit 1
fi
rm "${COOKIE_JAR_FAIL}"

# Test 9: Problems page
echo "Testing /problems..."
PROBLEMS_PAGE_RESPONSE=$(curl -s "${SERVER_URL}/problems")
if echo "${PROBLEMS_PAGE_RESPONSE}" | grep -q "<title>Problems" ; then
  echo -e "/problems \e[32mpassed\e[0m."
else
  echo -e "/problems \e[31mfailed\e[0m. Response snippet: ${PROBLEMS_PAGE_RESPONSE:0:200}... (Full response length: ${#PROBLEMS_PAGE_RESPONSE}) "
  exit 1
fi

# Test 10: Problem Statement page
echo "Testing /problems/graph_coloring..."
PROBLEM_STATEMENT_PAGE_RESPONSE=$(curl -s "${SERVER_URL}/problems/graph_coloring")

if echo "${PROBLEM_STATEMENT_PAGE_RESPONSE}" | grep -q "<title>Graph Coloring - solve.NP</title>"; then
  echo -e "/problems/graph_coloring \e[32mpassed\e[0m."
else
  echo -e "/problems/graph_coloring \e[31mfailed\e[0m. Response: ${PROBLEM_STATEMENT_PAGE_RESPONSE}"
  exit 1
fi

# Test 11: P vs NP page
echo "Testing /p_vs_np..."
P_VS_NP_PAGE_RESPONSE=$(curl -s "${SERVER_URL}/p_vs_np")
if echo "${P_VS_NP_PAGE_RESPONSE}" | grep -q "P vs NP"; then
  echo -e "/p_vs_np \e[32mpassed\e[0m."
else
  echo -e "/p_vs_np \e[31mfailed\e[0m. Response snippet: ${P_VS_NP_PAGE_RESPONSE:0:200}... (Full response length: ${#P_VS_NP_PAGE_RESPONSE}) "
  exit 1
fi

# Test 12: Logout
echo "Testing /log_out..."
# Need to log in first to get a session
COOKIE_JAR=$(mktemp)

# Perform the login POST request to get a session cookie
curl -s -D - -c "${COOKIE_JAR}" -X POST -d "username=user&password=pass" "${SERVER_URL}/api/log_in" -o /dev/null

# Perform the logout GET request and capture headers
LOGOUT_RESPONSE_HEADERS=$(curl -s -D - -b "${COOKIE_JAR}" "${SERVER_URL}/api/log_out" -o /dev/null)

# Check if the logout GET resulted in a 303 See Other redirect
if echo "${LOGOUT_RESPONSE_HEADERS}" | grep -q "HTTP/1.1 303 See Other"; then
  echo -e "Logout GET redirect status 303 \e[32mreceived\e[0m."
else
  echo -e "Logout GET \e[31mfailed\e[0m to redirect with 303. Headers: ${LOGOUT_RESPONSE_HEADERS}"
  rm "${COOKIE_JAR}"
  exit 1
fi

# Explicitly delete the cookie file after logout
rm "${COOKIE_JAR}"

# Fetch the home page after logout without sending any cookies
HOME_PAGE_AFTER_LOGOUT=$(curl -s "${SERVER_URL}/")

if echo "${HOME_PAGE_AFTER_LOGOUT}" | grep -q "Hello, Unknown user!"; then
  echo -e "Logout test \e[32mpassed\e[0m."
else
  echo -e "Logout test \e[31mfailed\e[0m. Home page content: ${HOME_PAGE_AFTER_LOGOUT:0:200}... (Full response length: ${#HOME_PAGE_AFTER_LOGOUT}) "
  exit 1
fi

# Test 13: Validate Sample Solution (TSP) - Correct
echo "Testing /api/validate_sample (TSP) - correct solution..."
TSP_SOLUTION='{"TspSolution":[0,1,4,2,6,5,3,7,0]}'
VALIDATE_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "${TSP_SOLUTION}" "${SERVER_URL}/api/validate_sample")

if echo "${VALIDATE_RESPONSE}" | jq -e '.Valid == 69' > /dev/null; then
    echo -e "/api/validate_sample (TSP) - correct \e[32mpassed\e[0m."
else
    echo -e "/api/validate_sample (TSP) - correct \e[31mfailed\e[0m. Response: ${VALIDATE_RESPONSE}"
    exit 1
fi

# Test 14: Validate Sample Solution (TSP) - Incorrect
echo "Testing /api/validate_sample (TSP) - incorrect solution..."
TSP_SOLUTION='{"TspSolution":[0,1,2,3]}'
VALIDATE_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "${TSP_SOLUTION}" "${SERVER_URL}/api/validate_sample")

if echo "${VALIDATE_RESPONSE}" | jq -e '.Invalid' > /dev/null; then
    echo -e "/api/validate_sample (TSP) - incorrect \e[32mpassed\e[0m."
else
    echo -e "/api/validate_sample (TSP) - incorrect \e[31mfailed\e[0m. Response: ${VALIDATE_RESPONSE}"
    exit 1
fi

# Test 15: Validate Sample Solution (Graph Coloring) - Incorrect
echo "Testing /api/validate_sample (Graph Coloring) - incorrect solution..."
GC_SOLUTION='{"GraphColoringSolution":[0,0,0,0,0,0,0,0]}'
VALIDATE_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "${GC_SOLUTION}" "${SERVER_URL}/api/validate_sample")

if echo "${VALIDATE_RESPONSE}" | jq -e '.Invalid' > /dev/null; then
    echo -e "/api/validate_sample (Graph Coloring) - incorrect \e[32mpassed\e[0m."
else
    echo -e "/api/validate_sample (Graph Coloring) - incorrect \e[31mfailed\e[0m. Response: ${VALIDATE_RESPONSE}"
    exit 1
fi


# Test 17: Vote endpoint - successful first vote 'eq' for 'user'
echo "Testing /api/vote/eq - successful first vote for 'user'..."
COOKIE_JAR_VOTE_USER=$(mktemp)

# Log in as 'user' to get a session cookie
curl -s -c "${COOKIE_JAR_VOTE_USER}" -X POST -d "username=user&password=pass" "${SERVER_URL}/api/log_in" > /dev/null

# Send the first vote request for 'user'
VOTE_EQ_RESPONSE_HEADERS=$(curl -s -D - -X POST -b "${COOKIE_JAR_VOTE_USER}" "${SERVER_URL}/api/vote/eq" -o /dev/null)

if echo "${VOTE_EQ_RESPONSE_HEADERS}" | grep -q "HTTP/1.1 303 See Other"; then
    echo -e "/api/vote/eq first vote for 'user' \e[32mpassed\e[0m."
else
    echo -e "/api/vote/eq first vote for 'user' \e[31mfailed\e[0m. Headers: ${VOTE_EQ_RESPONSE_HEADERS}"
    rm "${COOKIE_JAR_VOTE_USER}"
    exit 1
fi

# Test 18: Vote endpoint - failed second vote for 'user'
echo "Testing /api/vote/neq - failed second vote for 'user'..."

# Send the second vote request for 'user'
VOTE_NEQ_RESPONSE_CODE=$(curl -s -w "%{http_code}" -X POST -b "${COOKIE_JAR_VOTE_USER}" "${SERVER_URL}/api/vote/neq" -o /dev/null)

if [ "${VOTE_NEQ_RESPONSE_CODE}" -eq 500 ]; then
    echo -e "/api/vote/neq second vote for 'user' correctly \e[32mfailed\e[0m with status 500."
else
    echo -e "/api/vote/neq second vote for 'user' \e[31mfailed\e[0m. Expected 500, got ${VOTE_NEQ_RESPONSE_CODE}"
    rm "${COOKIE_JAR_VOTE_USER}"
    exit 1
fi
rm "${COOKIE_JAR_VOTE_USER}"

# Test 19: Vote endpoint - successful vote 'neq' for 'another_user'
echo "Testing /api/vote/neq - successful vote for 'another_user'..."
COOKIE_JAR_VOTE_ANOTHER=$(mktemp)

# Log in as 'another_user' to get a session cookie
curl -s -c "${COOKIE_JAR_VOTE_ANOTHER}" -X POST -d "username=another_user&password=pass" "${SERVER_URL}/api/log_in" > /dev/null

# Send the vote request for 'another_user'
VOTE_NEQ_RESPONSE_HEADERS=$(curl -s -D - -X POST -b "${COOKIE_JAR_VOTE_ANOTHER}" "${SERVER_URL}/api/vote/neq" -o /dev/null)

if echo "${VOTE_NEQ_RESPONSE_HEADERS}" | grep -q "HTTP/1.1 303 See Other"; then
    echo -e "/api/vote/neq for 'another_user' \e[32mpassed\e[0m."
else
    echo -e "/api/vote/neq for 'another_user' \e[31mfailed\e[0m. Headers: ${VOTE_NEQ_RESPONSE_HEADERS}"
    rm "${COOKIE_JAR_VOTE_ANOTHER}"
    exit 1
fi
rm "${COOKIE_JAR_VOTE_ANOTHER}"

# Test 20: Vote endpoint - invalid vote
echo "Testing /api/vote/invalid - invalid vote..."
COOKIE_JAR_VOTE_INVALID=$(mktemp)

# Log in to get a session cookie (user doesn't matter here)
curl -s -c "${COOKIE_JAR_VOTE_INVALID}" -X POST -d "username=user&password=pass" "${SERVER_URL}/api/log_in" > /dev/null

# Send the invalid vote request
INVALID_VOTE_RESPONSE=$(curl -s -w "%{http_code}" -X POST -b "${COOKIE_JAR_VOTE_INVALID}" "${SERVER_URL}/api/vote/invalid" -o /dev/null)

if [ "${INVALID_VOTE_RESPONSE}" -eq 400 ]; then
    echo -e "/api/vote/invalid \e[32mpassed\e[0m."
else
    echo -e "/api/vote/invalid \e[31mfailed\e[0m. HTTP status code: ${INVALID_VOTE_RESPONSE}"
    rm "${COOKIE_JAR_VOTE_INVALID}"
    exit 1
fi
rm "${COOKIE_JAR_VOTE_INVALID}"


# Test 21: Create Solution
echo "Testing /api/solutions (POST)..."
COOKIE_JAR_SOLUTION=$(mktemp)

# Log in as 'user'
curl -s -c "${COOKIE_JAR_SOLUTION}" -X POST -d "username=user&password=pass" "${SERVER_URL}/api/log_in" > /dev/null

# Create a solution
CREATE_SOLUTION_RESPONSE=$(curl -s -X POST -b "${COOKIE_JAR_SOLUTION}" -H "Content-Type: application/json" -d '{"problem_id":"graph_coloring","title":"My first solution","description":"A description"}' "${SERVER_URL}/api/solutions")

# After creating the solution, get the solution_id from the /solutions page
SOLUTIONS_PAGE_HTML=$(curl -s -b "${COOKIE_JAR_SOLUTION}" "${SERVER_URL}/solutions")
SOLUTION_ID=$(echo "${SOLUTIONS_PAGE_HTML}" | grep -oP "[0-9a-f]{32}" | head -n 1)

if [ -n "${SOLUTION_ID}" ] && [ "${SOLUTION_ID}" != "null" ]; then
    echo -e "/api/solutions (POST) \e[32mpassed\e[0m. Solution ID: ${SOLUTION_ID}"
else
    echo -e "/api/solutions (POST) \e[31mfailed\e[0m. Could not retrieve Solution ID from /solutions page. Response: ${SOLUTIONS_PAGE_HTML}"
    rm "${COOKIE_JAR_SOLUTION}"
    exit 1
fi

# Test 22: Delete Solution
echo "Testing /api/solutions/{solution_id} (DELETE)..."
DELETE_SOLUTION_RESPONSE_HEADERS=$(curl -s -D - -X DELETE -b "${COOKIE_JAR_SOLUTION}" "${SERVER_URL}/api/solutions/${SOLUTION_ID}" -o /dev/null)


if echo "${DELETE_SOLUTION_RESPONSE_HEADERS}" | grep -q "HTTP/1.1 204 No Content"; then
    echo -e "/api/solutions/{solution_id} (DELETE) \e[32mpassed\e[0m."
else
    echo -e "/api/solutions/{solution_id} (DELETE) \e[31mfailed\e[0m. Headers: ${DELETE_SOLUTION_RESPONSE_HEADERS}"
    rm "${COOKIE_JAR_SOLUTION}"
    exit 1
fi
rm "${COOKIE_JAR_SOLUTION}"

# Test 23: Submit Solution
echo "Testing /api/submit_solution (POST)..."
COOKIE_JAR_SUBMISSION=$(mktemp)

# Log in as 'user'
curl -s -c "${COOKIE_JAR_SUBMISSION}" -X POST -d "username=user&password=pass" "${SERVER_URL}/api/log_in" > /dev/null

# Create a solution
CREATE_SOLUTION_RESPONSE=$(curl -s -X POST -b "${COOKIE_JAR_SUBMISSION}" -H "Content-Type: application/json" -d '{"problem_id":"graph_coloring","title":"My submission solution","description":"A description"}' "${SERVER_URL}/api/solutions")
SOLUTIONS_PAGE_HTML=$(curl -s -b "${COOKIE_JAR_SUBMISSION}" "${SERVER_URL}/solutions")
SOLUTION_ID=$(echo "${SOLUTIONS_PAGE_HTML}" | grep -oP "[0-9a-f]{32}" | head -n 1)

# Submit a solution
SUBMIT_SOLUTION_RESPONSE=$(curl -s -X POST -b "${COOKIE_JAR_SUBMISSION}" -H "Content-Type: application/json" -d '{"GraphColoringSolution":[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99]}' "${SERVER_URL}/api/submit_solution/graph_coloring/${SOLUTION_ID}/small_ultra_sparse")

if echo "${SUBMIT_SOLUTION_RESPONSE}" | jq -e '.Valid' > /dev/null; then
    echo -e "/api/submit_solution (POST) \e[32mpassed\e[0m."
else
    echo -e "/api/submit_solution (POST) \e[31mfailed\e[0m. Response: ${SUBMIT_SOLUTION_RESPONSE}"
    rm "${COOKIE_JAR_SUBMISSION}"
    exit 1
fi
rm "${COOKIE_JAR_SUBMISSION}"



# Test 24: Solution Status
echo "Testing /solutions/{solution_id}/status..."
COOKIE_JAR_SOLUTION_STATUS=$(mktemp)

# Log in as 'user'
curl -s -c "${COOKIE_JAR_SOLUTION_STATUS}" -X POST -d "username=user&password=pass" "${SERVER_URL}/api/log_in" > /dev/null


# Create a solution
CREATE_SOLUTION_RESPONSE=$(curl -s -X POST -b "${COOKIE_JAR_SOLUTION_STATUS}" -H "Content-Type: application/json" -d '{"problem_id":"graph_coloring","title":"Solution for status test","description":"A description"}' "${SERVER_URL}/api/solutions")
SOLUTIONS_PAGE_HTML=$(curl -s -b "${COOKIE_JAR_SOLUTION_STATUS}" "${SERVER_URL}/solutions")
SOLUTION_ID_STATUS=$(echo "${SOLUTIONS_PAGE_HTML}" | grep "Solution for status test" | grep -oP "[0-9a-f]{32}" | head -n 1)

if [ -z "${SOLUTION_ID_STATUS}" ]; then
    echo -e "/solutions/{solution_id}/status \e[31mfailed\e[0m. Could not retrieve Solution ID."
    rm "${COOKIE_JAR_SOLUTION_STATUS}"
    exit 1
fi

SOLUTION_STATUS_RESPONSE=$(curl -s -b "${COOKIE_JAR_SOLUTION_STATUS}" "${SERVER_URL}/api/solutions/${SOLUTION_ID_STATUS}/status")

if [ "${SOLUTION_STATUS_RESPONSE}" == "\"Untested\"" ]; then
    echo -e "/solutions/{solution_id}/status \e[32mpassed\e[0m."
else
    echo -e "/solutions/{solution_id}/status \e[31mfailed\e[0m. Response: ${SOLUTION_STATUS_RESPONSE}"
    rm "${COOKIE_JAR_SOLUTION_STATUS}"
    exit 1
fi

# Test 25: Solution Input Status
echo "Testing /solutions/{solution_id}/input/{input_id}/status..."
INPUT_ID="small_ultra_sparse" # Using an input_id from existing tests

SOLUTION_INPUT_STATUS_RESPONSE=$(curl -s -b "${COOKIE_JAR_SOLUTION_STATUS}" "${SERVER_URL}/api/solutions/${SOLUTION_ID}/input/${INPUT_ID}/status")

if [ "${SOLUTION_INPUT_STATUS_RESPONSE}" == "{\"Solved\":100}" ]; then
    echo -e "/solutions/{solution_id}/input/{input_id}/status \e[32mpassed\e[0m."
else
    echo -e "/solutions/{solution_id}/input/{input_id}/status \e[31mfailed\e[0m. Response: ${SOLUTION_INPUT_STATUS_RESPONSE}"
    rm "${COOKIE_JAR_SOLUTION_STATUS}"
    exit 1
fi
rm "${COOKIE_JAR_SOLUTION_STATUS}"

echo -e "\e[32mAll tests passed!\e[0m"
rm -f "${DB_PATH}"
rm -f test.log
