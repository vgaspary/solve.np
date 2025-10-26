#!/bin/bash
if [ ! -f ../solve_np.db3 ]; then
  echo "Creating and setting up the DB."
  sqlite3 ../solve_np.db3 < ../init.sql
fi

cargo watch -i "js/dist/bundle.js" -i "../solve_np.db3" -s "npx esbuild js/src/index.ts --outfile=js/dist/bundle.js --bundle; cargo r -- -s static_data.yaml -k 4M/q/i1+d9eQXO3Bzn0L4sRWBrJ3cpCschtUAsC22Z4KrbVhXiVpBMBJnhk+pG85qy5Ra1IwkvPyXTtlAyleuw== -d ../solve_np.db3 -p 3333"
