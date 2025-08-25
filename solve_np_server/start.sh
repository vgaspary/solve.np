#!/bin/bash
if [ ! -f ../solve_np.db3 ]; then
  sqlite3 ../solve_np.db3 "CREATE TABLE votes (id INTEGER PRIMARY KEY AUTOINCREMENT, user_id TEXT UNIQUE, vote BOOLEAN);"
fi

cargo watch -i "js_utils/*.js" -i "../solve_np.db3" -s "tsc js_utils/*.ts; cargo r -- -s static_data.yaml -k 4M/q/i1+d9eQXO3Bzn0L4sRWBrJ3cpCschtUAsC22Z4KrbVhXiVpBMBJnhk+pG85qy5Ra1IwkvPyXTtlAyleuw== -d ../solve_np.db3"