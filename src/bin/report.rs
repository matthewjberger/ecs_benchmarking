use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const SCENARIOS: &[(&str, &str)] = &[
    ("simple_insert", "Insert 10K entities, 4 components each"),
    ("simple_iter", "Iterate 10K entities, position += velocity"),
    (
        "fragmented_iter",
        "Iterate Data across 26 fragmented archetypes",
    ),
    (
        "heavy_compute",
        "Parallel matrix inversion over 1K entities",
    ),
    ("add_remove", "Add then remove a component on 10K entities"),
    ("schedule", "Run a 3-system schedule over 40K entities"),
];

#[derive(Deserialize)]
struct Estimate {
    point_estimate: f64,
}

#[derive(Deserialize)]
struct Estimates {
    mean: Estimate,
}

struct Row {
    description: &'static str,
    freecs_ns: Option<f64>,
    bevy_ns: Option<f64>,
}

fn criterion_home() -> String {
    std::env::var("CRITERION_HOME").unwrap_or_else(|_| "target/criterion".to_string())
}

fn read_mean(group: &str, function: &str) -> Option<f64> {
    let path = format!(
        "{}/{}/{}/new/estimates.json",
        criterion_home(),
        group,
        function
    );
    let text = fs::read_to_string(path).ok()?;
    let estimates: Estimates = serde_json::from_str(&text).ok()?;
    Some(estimates.mean.point_estimate)
}

fn format_time(nanos: f64) -> String {
    if nanos < 1_000.0 {
        format!("{nanos:.1} ns")
    } else if nanos < 1_000_000.0 {
        format!("{:.2} us", nanos / 1_000.0)
    } else if nanos < 1_000_000_000.0 {
        format!("{:.2} ms", nanos / 1_000_000.0)
    } else {
        format!("{:.2} s", nanos / 1_000_000_000.0)
    }
}

fn utc_timestamp(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = (secs % 86_400) as i64;
    let hour = rem / 3_600;
    let minute = (rem % 3_600) / 60;
    let second = rem % 60;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

fn winner(row: &Row) -> Option<(&'static str, f64)> {
    match (row.freecs_ns, row.bevy_ns) {
        (Some(freecs), Some(bevy)) => {
            if freecs <= bevy {
                Some(("freecs", bevy / freecs))
            } else {
                Some(("bevy", freecs / bevy))
            }
        }
        _ => None,
    }
}

fn pad(text: &str, width: usize) -> String {
    let mut out = text.to_string();
    while out.chars().count() < width {
        out.push(' ');
    }
    out
}

fn print_terminal_table(rows: &[Row]) {
    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const DIM: &str = "\x1b[2m";
    const GREEN: &str = "\x1b[32m";
    const CYAN: &str = "\x1b[36m";

    let scenario_width = rows
        .iter()
        .map(|row| row.description.chars().count())
        .max()
        .unwrap_or(20)
        .max("Scenario".len());
    let col = 12;

    let header = format!(
        "  {}  {}  {}  {}  {}",
        pad("Scenario", scenario_width),
        pad("freecs", col),
        pad("bevy", col),
        pad("winner", 8),
        "speedup",
    );
    println!();
    println!("{BOLD}{CYAN}  freecs vs bevy_ecs{RESET}");
    println!("{DIM}{header}{RESET}");
    println!("  {}", "-".repeat(scenario_width + col * 2 + 8 + 10 + 8));

    for row in rows {
        let freecs = row.freecs_ns.map(format_time).unwrap_or_else(|| "-".into());
        let bevy = row.bevy_ns.map(format_time).unwrap_or_else(|| "-".into());
        match winner(row) {
            Some((who, ratio)) => {
                let freecs_cell = if who == "freecs" {
                    format!("{GREEN}{}{RESET}", pad(&freecs, col))
                } else {
                    pad(&freecs, col)
                };
                let bevy_cell = if who == "bevy" {
                    format!("{GREEN}{}{RESET}", pad(&bevy, col))
                } else {
                    pad(&bevy, col)
                };
                println!(
                    "  {}  {}  {}  {}{}{}  {:.2}x",
                    pad(row.description, scenario_width),
                    freecs_cell,
                    bevy_cell,
                    GREEN,
                    pad(who, 8),
                    RESET,
                    ratio,
                );
            }
            None => {
                println!(
                    "  {}  {}  {}  {}  -",
                    pad(row.description, scenario_width),
                    pad(&freecs, col),
                    pad(&bevy, col),
                    pad("-", 8),
                );
            }
        }
    }
    println!();
}

fn markdown_report(rows: &[Row], timestamp: &str) -> String {
    let mut out = String::new();
    out.push_str("# freecs vs bevy_ecs\n\n");
    out.push_str(&format!("_Generated {timestamp}._\n\n"));
    out.push_str(
        "Times are Criterion mean estimates (lower is better). \
         Speedup is the ratio between the two implementations for that scenario.\n\n",
    );
    out.push_str("| Scenario | freecs | bevy_ecs | Winner | Speedup |\n");
    out.push_str("| --- | ---: | ---: | :---: | ---: |\n");

    for row in rows {
        let freecs = row.freecs_ns.map(format_time).unwrap_or_else(|| "-".into());
        let bevy = row.bevy_ns.map(format_time).unwrap_or_else(|| "-".into());
        let (winner_cell, speedup_cell) = match winner(row) {
            Some((who, ratio)) => (who.to_string(), format!("{ratio:.2}x")),
            None => ("-".to_string(), "-".to_string()),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            row.description, freecs, bevy, winner_cell, speedup_cell
        ));
    }

    out.push_str("\n## Scenario definitions\n\n");
    for (id, description) in SCENARIOS {
        out.push_str(&format!("- **`{id}`**: {description}\n"));
    }
    out.push_str("\nDetailed per-benchmark plots live under `target/criterion/`.\n");
    out
}

fn main() {
    let home = criterion_home();
    if !Path::new(&home).exists() {
        eprintln!(
            "No Criterion results found at `{home}`.\nRun `just bench` (or `cargo bench`) first."
        );
        std::process::exit(1);
    }

    let rows: Vec<Row> = SCENARIOS
        .iter()
        .map(|(id, description)| Row {
            description,
            freecs_ns: read_mean(id, "freecs"),
            bevy_ns: read_mean(id, "bevy"),
        })
        .collect();

    if rows
        .iter()
        .all(|row| row.freecs_ns.is_none() && row.bevy_ns.is_none())
    {
        eprintln!(
            "Criterion results exist but no matching scenarios were found. Run `just bench`."
        );
        std::process::exit(1);
    }

    print_terminal_table(&rows);

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let timestamp = utc_timestamp(secs);
    let markdown = markdown_report(&rows, &timestamp);

    if let Err(error) = fs::create_dir_all("reports") {
        eprintln!("Failed to create reports directory: {error}");
        std::process::exit(1);
    }
    let latest = "reports/latest.md";
    let stamped = format!("reports/report-{secs}.md");
    if let Err(error) = fs::write(latest, &markdown) {
        eprintln!("Failed to write {latest}: {error}");
        std::process::exit(1);
    }
    let _ = fs::write(&stamped, &markdown);

    println!("  Markdown report written to {latest} and {stamped}\n");
}
