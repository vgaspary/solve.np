CREATE TABLE
  votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT UNIQUE,
    vote BOOLEAN
  );

CREATE TABLE
  solutions (
    solution_id TEXT PRIMARY KEY,
    user_id TEXT,
    problem_id TEXT,
    title TEXT,
    description TEXT,
    status INTEGER
  );

CREATE UNIQUE INDEX idx_user_problem_title ON solutions (user_id, problem_id, title);

CREATE TABLE
  submissions (
    user_id TEXT,
    problem_id TEXT,
    solution_id TEXT,
    input_id TEXT,
    solution TEXT,
    hash_digest INTEGER,
    status INTEGER,
    invalid_message TEXT,
    PRIMARY KEY (user_id, problem_id, solution_id, input_id)
  );

CREATE TRIGGER update_solution_status_on_new_submission AFTER
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

CREATE TABLE
  num_inputs (problem_id TEXT, cnt INTEGER);

INSERT INTO
  num_inputs (problem_id, cnt)
VALUES
  ('graph_coloring', 9),
  ('travelling_salesman', 9);
