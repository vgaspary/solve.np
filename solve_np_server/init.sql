CREATE TABLE IF NOT EXISTS
  votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT UNIQUE NOT NULL,
    vote BOOLEAN NOT NULL
  );

CREATE TABLE IF NOT EXISTS
  solutions (
    solution_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    user_name TEXT NOT NULL,
    problem_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status INTEGER NOT NULL,
    ranking INTEGER
  );

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_problem_title ON solutions (user_id, problem_id, title);

CREATE TABLE IF NOT EXISTS
  submissions (
    user_id TEXT NOT NULL,
    user_name TEXT NOT NULL,
    problem_id TEXT NOT NULL,
    solution_id TEXT NOT NULL,
    input_id TEXT NOT NULL,
    solution TEXT NOT NULL,
    hash_digest INTEGER NOT NULL,
    status INTEGER NOT NULL,
    invalid_message TEXT NOT NULL,
    ranking INTEGER,
    PRIMARY KEY (user_id, problem_id, solution_id, input_id)
  );

CREATE TRIGGER IF NOT EXISTS update_solution_status_on_new_submission AFTER
UPDATE OF solution_id,
status ON submissions FOR EACH ROW WHEN OLD.status <> NEW.status BEGIN
UPDATE solutions
SET
  status = (
    SELECT
      CASE
        WHEN MAX(T1.status) = -2 THEN -2
        WHEN MIN(T1.status) >= 0
        AND COUNT(T1.status) = (
          SELECT
            cnt
          FROM
            num_inputs
          WHERE
            problem_id = NEW.problem_id
        ) THEN 0
        ELSE -1
      END
    FROM
      submissions AS T1
    WHERE
      T1.solution_id = NEW.solution_id
  )
WHERE
  solution_id = NEW.solution_id;

END;

CREATE TABLE IF NOT EXISTS
  num_inputs (problem_id TEXT UNIQUE, cnt INTEGER);

INSERT OR REPLACE INTO
  num_inputs (problem_id, cnt)
VALUES
  ('travelling_salesman', 9),
  ('graph_coloring', 9),
  ('vertex_cover', 9),
  ('maximum_clique', 9),
  ('maximum_cut', 12),
  ('steiner_tree', 12),
  ('knapsack', 8);
