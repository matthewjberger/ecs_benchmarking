set windows-shell := ["powershell.exe"]

# Displays the list of available commands
@just:
    just --list

# Run every benchmark, then print the comparison table and write reports/latest.md
run:
    cargo bench --bench benchmarks
    cargo run --release --quiet --bin report

# Run the benchmarks only (Criterion terminal output plus HTML under target/criterion)
bench:
    cargo bench --bench benchmarks

# Run a single scenario, e.g. `just bench-one simple_iter`
bench-one scenario:
    cargo bench --bench benchmarks -- {{scenario}}

# Rebuild the comparison report from the most recent benchmark run
report:
    cargo run --release --quiet --bin report

# Run the iteration-API experiment (freecs index vs raw slices vs sky_ecs)
experiment:
    cargo bench --bench experiment_iter

# cargo check plus a format check
check:
    cargo check --all-targets
    cargo fmt --all -- --check

# Format the code
format:
    cargo fmt --all

# Clippy with warnings denied
lint:
    cargo clippy --all-targets -- -D warnings

# Apply clippy fixes
fix:
    cargo clippy --all-targets --fix --allow-dirty

# Remove build artifacts and generated reports
clean:
    cargo clean
    if (Test-Path reports) { Remove-Item -Recurse -Force reports }
