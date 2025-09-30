#!/bin/bash
if [ ! -f ../solve_np.db3 ]; then
  echo "Creating and setting up the DB."
  sqlite3 ../solve_np.db3 "CREATE TABLE votes (id INTEGER PRIMARY KEY AUTOINCREMENT, user_id TEXT UNIQUE, vote BOOLEAN);" || exit 1
  sqlite3 ../solve_np.db3 "CREATE TABLE attempts (solution_id TEXT PRIMARY KEY, user_id TEXT, problem_id TEXT, title TEXT, description TEXT, status INTEGER);" || exit 1
  sqlite3 ../solve_np.db3 "CREATE UNIQUE INDEX idx_user_problem_title ON attempts (user_id, problem_id, title);" || exit 1
fi

cargo watch -i "js_utils/*.js" -i "../solve_np.db3" -s "tsc --project js_utils/tsconfig.json; cargo r -- -s static_data.yaml -k 4M/q/i1+d9eQXO3Bzn0L4sRWBrJ3cpCschtUAsC22Z4KrbVhXiVpBMBJnhk+pG85qy5Ra1IwkvPyXTtlAyleuw== -d ../solve_np.db3 -p 3333"
