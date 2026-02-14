# solvenp.now

This project is a web server for hosting a competition to solve NP-complete problems. It includes a Rust backend using `actix-web` and a Javascript frontend.

## Project Structure

The project is organized into the following main directories:

- `problems/`: Contains subdirectories for each NP-complete problem. Each problem has:
    - A problem statement in `problem_statement.md`.
    - A generator for creating new instances of the problem (`<problem>_gen/`).
    - A validator for solutions (`<problem>_validate/`).
    - Sample inputs in the `inputs/` directory.
- `solve_np_server/`: The main web server application.
    - `src/`: The Rust source code for the server.
    - `js/`: The Javascript frontend code.
    - `static/`: Static assets like images and CSS.
    - `templates/`: HTML templates for the frontend.

## Building the Project

The project is built using `cargo` for the Rust backend and `npx esbuild` for the Javascript frontend. The `solve_np_server/build.rs` script orchestrates the build process.

To build the entire project run:

```bash
cargo build
```

This command will:
1. Compile the Rust code for the server and all `problem` crates.
2.  Build the Javascript bundle using `esbuild`.
3.  Create an SQLite database file `solve_np.db3` and initialize it with the schema from `init.sql`.

## Running the Project Locally

To run the server locally, you first need to set a secret key for signing session cookies. This should be a base64 encoded string. You can generate one with:

```bash
openssl rand -base64 64
```

Then, you can run the server from the `solve_np_server` directory with the following command, replacing `<your-key>` with the key you generated:

```bash
cargo run -- --key <your-key>
```

The server will start on `http://0.0.0.0:8080` by default. You can change the port with the `--port` argument.

For hot reloading you can use `cargo watch`:

```bash
cargo watch -i "js/dist/bundle.js" -i "solve_np.db3*" -i "session.yaml" -- cargo run -- --key <your-key>
```

## Testing the Project

The project includes unit tests for different modules, as well as end-to-end tests that build and run the server and test its functionality. To run the tests simply run:

```bash
cargo test
```

This will run all tests, including the end-to-end tests in `tests/e2e_tests.rs`. Note: upon failing end-to-end tests leave some artifacts behind in `solve_np_server/` for inspection, an SQLite DB file and a session file.
