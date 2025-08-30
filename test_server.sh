#!/bin/bash

set -euo pipefail

ROOT_DIR=$(pwd)
SERVER_DIR="${ROOT_DIR}/solve_np_server"
STATIC_DATA_PATH="${SERVER_DIR}/static_data.yaml"
DB_PATH="${SERVER_DIR}/test_file.db3"
SERVER_ADDRESS="127.0.0.1"
SERVER_PORT="6520"
SERVER_URL="http://${SERVER_ADDRESS}:${SERVER_PORT}"

# Create the database and the votes table
rm -f "${DB_PATH}"
sqlite3 "${DB_PATH}" "CREATE TABLE votes (id INTEGER PRIMARY KEY AUTOINCREMENT, user_id TEXT UNIQUE, vote BOOLEAN);"

# Generate a random base64 encoded key for the session middleware
RANDOM_KEY=$(head /dev/urandom | tr -dc A-Za-z0-9_ | head -c 64 | base64 -w 0)

echo "Building solve_np_server..."
cd "${SERVER_DIR}" && cargo build

echo "Starting solve_np_server in the background..."
# Start the server and redirect output to a log file
"${ROOT_DIR}/target/debug/solve_np_server" \
  --static-data-path "${STATIC_DATA_PATH}" \
  --db-path "${DB_PATH}" \
  --key "${RANDOM_KEY}" > test.log 2>&1 &

SERVER_PID=$!
echo "Server started with PID: ${SERVER_PID}"

# Function to clean up server process on exit
cleanup() {
  echo "Stopping server (PID: ${SERVER_PID})..."
  kill -9 "${SERVER_PID}" || true # Use || true to prevent script from exiting if kill fails
  wait "${SERVER_PID}" 2>/dev/null || true # Wait for the process to terminate
  rm -f "${DB_PATH}"
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

# Test 2: List Problems API
echo "Testing /api/problems..."
PROBLEMS_RESPONSE=$(curl -s "${SERVER_URL}/api/problems")
if echo "${PROBLEMS_RESPONSE}" | jq -e '.[].id' | grep -q "graph_coloring" && echo "${PROBLEMS_RESPONSE}" | jq -e '.[].id' | grep -q "travelling_salesman"; then
  echo -e "/api/problems \e[32mpassed\e[0m."
else
  echo -e "/api/problems \e[31mfailed\e[0m. Response: ${PROBLEMS_RESPONSE}"
  exit 1
fi

# Test 3: Get Problem Details API
echo "Testing /api/problems/graph_coloring..."
PROBLEM_DETAILS_RESPONSE=$(curl -s "${SERVER_URL}/api/problems/graph_coloring")
if echo "${PROBLEM_DETAILS_RESPONSE}" | jq -e '.id == "graph_coloring" and .name == "Graph Coloring" and .problem_statement_html' > /dev/null; then
  echo -e "/api/problems/graph_coloring \e[32mpassed\e[0m."
else
  echo -e "/api/problems/graph_coloring \e[31mfailed\e[0m. Response: ${PROBLEM_DETAILS_RESPONSE}"
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

if echo "${PROBLEM_STATEMENT_PAGE_RESPONSE}" | grep -q "<article id=\"problem_statement\">"; then
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


# Test 17: Vote endpoint - successful vote 'eq'
echo "Testing /api/vote/eq - successful vote..."
COOKIE_JAR_VOTE=$(mktemp)

# Log in to get a session cookie
curl -s -c "${COOKIE_JAR_VOTE}" -X POST -d "username=user&password=pass" "${SERVER_URL}/api/log_in" > /dev/null

# Send the vote request
VOTE_EQ_RESPONSE_HEADERS=$(curl -s -D - -X POST -b "${COOKIE_JAR_VOTE}" "${SERVER_URL}/api/vote/eq" -o /dev/null)

if echo "${VOTE_EQ_RESPONSE_HEADERS}" | grep -q "HTTP/1.1 303 See Other"; then
    echo -e "/api/vote/eq \e[32mpassed\e[0m."
else
    echo -e "/api/vote/eq \e[31mfailed\e[0m. Headers: ${VOTE_EQ_RESPONSE_HEADERS}"
    rm "${COOKIE_JAR_VOTE}"
    exit 1
fi
rm "${COOKIE_JAR_VOTE}"

# Test 18: Vote endpoint - successful vote 'neq'
echo "Testing /api/vote/neq - successful vote..."
COOKIE_JAR_VOTE=$(mktemp)

# Log in to get a session cookie
curl -s -c "${COOKIE_JAR_VOTE}" -X POST -d "username=another_user&password=pass" "${SERVER_URL}/api/log_in" > /dev/null

# Send the vote request
VOTE_NEQ_RESPONSE_HEADERS=$(curl -s -D - -X POST -b "${COOKIE_JAR_VOTE}" "${SERVER_URL}/api/vote/neq" -o /dev/null)

if echo "${VOTE_NEQ_RESPONSE_HEADERS}" | grep -q "HTTP/1.1 303 See Other"; then
    echo -e "/api/vote/neq \e[32mpassed\e[0m."
else
    echo -e "/api/vote/neq \e[31mfailed\e[0m. Headers: ${VOTE_NEQ_RESPONSE_HEADERS}"
    rm "${COOKIE_JAR_VOTE}"
    exit 1
fi
rm "${COOKIE_JAR_VOTE}"

# Test 19: Vote endpoint - invalid vote
echo "Testing /api/vote/invalid - invalid vote..."
COOKIE_JAR_VOTE=$(mktemp)

# Log in to get a session cookie
curl -s -c "${COOKIE_JAR_VOTE}" -X POST -d "username=user&password=pass" "${SERVER_URL}/api/log_in" > /dev/null

# Send the vote request
INVALID_VOTE_RESPONSE=$(curl -s -w "%{http_code}" -X POST -b "${COOKIE_JAR_VOTE}" "${SERVER_URL}/api/vote/invalid" -o /dev/null)

if [ "${INVALID_VOTE_RESPONSE}" -eq 400 ]; then
    echo -e "/api/vote/invalid \e[32mpassed\e[0m."
else
    echo -e "/api/vote/invalid \e[31mfailed\e[0m. HTTP status code: ${INVALID_VOTE_RESPONSE}"
    rm "${COOKIE_JAR_VOTE}"
    exit 1
fi
rm "${COOKIE_JAR_VOTE}"

echo -e "\e[32mAll tests passed!\e[0m"
rm -f test.log
