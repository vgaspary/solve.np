#!/bin/bash
if [ ! -f ../solve_np.db3 ]; then
  echo "Creating and setting up the DB."

  sqlite3 ../solve_np.db3 \
    "CREATE TABLE votes (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      user_id TEXT UNIQUE,
      vote BOOLEAN
      );" || exit 1

  sqlite3 ../solve_np.db3 \
    "CREATE TABLE solutions (
      solution_id TEXT PRIMARY KEY,
      user_id TEXT,
      problem_id TEXT,
      title TEXT,
      description TEXT,
      status INTEGER
      );" || exit 1
  sqlite3 ../solve_np.db3 "CREATE UNIQUE INDEX idx_user_problem_title ON solutions (user_id, problem_id, title);" || exit 1

  sqlite3 ../solve_np.db3 \
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
      );" || exit 1

  sqlite3 ../solve_np.db3 \
    "CREATE TRIGGER update_solution_status_on_new_submission
     AFTER UPDATE OF solution_id, status ON submissions
     FOR EACH ROW
     WHEN OLD.status <> NEW.status
     BEGIN
       UPDATE solutions
       SET status = (
         SELECT
           CASE
             WHEN MAX(T1.status) = -2 THEN -2
             WHEN MIN(T1.status) >= 0 AND COUNT(T1.status) = (SELECT count FROM num_inputs WHERE problem_id = NEW.problem_id) THEN 0
             ELSE -1
           END
         FROM submissions AS T1
         WHERE T1.solution_id = NEW.solution_id
       )
       WHERE solution_id = NEW.solution_id;
     END;" || exit 1

  sqlite3 ../solve_np.db3 \
    "CREATE TABLE num_inputs (
      problem_id TEXT,
      count INTEGER
    );

    INSERT INTO num_inputs (problem_id, count) VALUES
    ('graph_coloring', 9),
    ('travelling_salesman', 9);"
fi

cargo watch -i "js_utils/*.js" -i "../solve_np.db3" -s "tsc --project js_utils/tsconfig.json; cargo r -- -s static_data.yaml -k 4M/q/i1+d9eQXO3Bzn0L4sRWBrJ3cpCschtUAsC22Z4KrbVhXiVpBMBJnhk+pG85qy5Ra1IwkvPyXTtlAyleuw== -d ../solve_np.db3 -p 3333"
