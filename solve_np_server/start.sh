#!/bin/bash
cargo watch -i "js/dist/bundle.js" -i "solve_np.db3" -- cargo r -- -s static_data.yaml -k 4M/q/i1+d9eQXO3Bzn0L4sRWBrJ3cpCschtUAsC22Z4KrbVhXiVpBMBJnhk+pG85qy5Ra1IwkvPyXTtlAyleuw== -d solve_np.db3 -p 3333
