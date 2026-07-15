use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
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

// Columns are laid out in this order when present; any other discovered
// library is appended alphabetically. Adding a library needs no change here.
const PREFERRED_ORDER: &[&str] = &["freecs", "freecs_dyn", "bevy", "skyecs"];

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
    values: BTreeMap<String, f64>,
}

impl Row {
    fn cell(&self, library: &str) -> String {
        self.values
            .get(library)
            .map(|nanos| format_time(*nanos))
            .unwrap_or_else(|| "-".into())
    }

    fn fastest(&self) -> Option<&String> {
        self.values
            .iter()
            .min_by(|left, right| left.1.total_cmp(right.1))
            .map(|(library, _)| library)
    }

    fn speedup_over_next(&self) -> Option<f64> {
        let mut times: Vec<f64> = self.values.values().copied().collect();
        if times.len() < 2 {
            return None;
        }
        times.sort_by(|left, right| left.total_cmp(right));
        Some(times[1] / times[0])
    }
}

fn criterion_home() -> String {
    std::env::var("CRITERION_HOME").unwrap_or_else(|_| "target/criterion".to_string())
}

fn read_mean(path: &str) -> Option<f64> {
    let text = fs::read_to_string(path).ok()?;
    let estimates: Estimates = serde_json::from_str(&text).ok()?;
    Some(estimates.mean.point_estimate)
}

fn discover(group: &str) -> BTreeMap<String, f64> {
    let dir = format!("{}/{}", criterion_home(), group);
    let mut results = BTreeMap::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return results;
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            continue;
        }
        let library = entry.file_name().to_string_lossy().to_string();
        let estimates = format!("{dir}/{library}/new/estimates.json");
        if let Some(nanos) = read_mean(&estimates) {
            results.insert(library, nanos);
        }
    }
    results
}

fn order_libraries(all: &BTreeSet<String>) -> Vec<String> {
    let mut ordered: Vec<String> = PREFERRED_ORDER
        .iter()
        .filter(|library| all.contains(**library))
        .map(|library| library.to_string())
        .collect();
    for library in all {
        if !ordered.contains(library) {
            ordered.push(library.clone());
        }
    }
    ordered
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

fn pad(text: &str, width: usize) -> String {
    let mut out = text.to_string();
    while out.chars().count() < width {
        out.push(' ');
    }
    out
}

fn print_terminal_table(order: &[String], rows: &[Row]) {
    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const DIM: &str = "\x1b[2m";
    const GREEN: &str = "\x1b[32m";
    const CYAN: &str = "\x1b[36m";

    let scenario_width = rows
        .iter()
        .map(|row| row.description.chars().count())
        .chain(std::iter::once("Scenario".len()))
        .max()
        .unwrap_or(20);

    let mut widths: Vec<usize> = order.iter().map(|library| library.len().max(10)).collect();
    for row in rows {
        for (index, library) in order.iter().enumerate() {
            widths[index] = widths[index].max(row.cell(library).chars().count());
        }
    }
    let fastest_width = order
        .iter()
        .map(|library| library.len())
        .max()
        .unwrap_or(7)
        .max(7);

    print!("\n{BOLD}{CYAN}  ");
    print!("{}", order.join(" vs "));
    println!("{RESET}");

    let mut header = format!("  {}", pad("Scenario", scenario_width));
    for (index, library) in order.iter().enumerate() {
        header.push_str(&format!("  {}", pad(library, widths[index])));
    }
    header.push_str(&format!("  {}  speedup", pad("fastest", fastest_width)));
    println!("{DIM}{header}{RESET}");

    let total: usize =
        scenario_width + widths.iter().map(|width| width + 2).sum::<usize>() + fastest_width + 13;
    println!("  {}", "-".repeat(total));

    for row in rows {
        let fastest = row.fastest();
        let mut line = format!("  {}", pad(row.description, scenario_width));
        for (index, library) in order.iter().enumerate() {
            let cell = pad(&row.cell(library), widths[index]);
            if Some(library) == fastest {
                line.push_str(&format!("  {GREEN}{cell}{RESET}"));
            } else {
                line.push_str(&format!("  {cell}"));
            }
        }
        match (fastest, row.speedup_over_next()) {
            (Some(library), Some(ratio)) => {
                line.push_str(&format!(
                    "  {GREEN}{}{RESET}  {ratio:.2}x",
                    pad(library, fastest_width)
                ));
            }
            (Some(library), None) => {
                line.push_str(&format!(
                    "  {GREEN}{}{RESET}  -",
                    pad(library, fastest_width)
                ));
            }
            _ => {
                line.push_str(&format!("  {}  -", pad("-", fastest_width)));
            }
        }
        println!("{line}");
    }
    println!();
}

fn markdown_report(order: &[String], rows: &[Row], timestamp: &str) -> String {
    let mut out = String::new();
    out.push_str("# ECS benchmark comparison\n\n");
    out.push_str(&format!("_Generated {timestamp}._\n\n"));
    out.push_str(
        "Times are Criterion mean estimates (lower is better). Speedup is how much \
         faster the winner is than the next-fastest implementation.\n\n",
    );

    out.push_str("| Scenario |");
    for library in order {
        out.push_str(&format!(" {library} |"));
    }
    out.push_str(" Fastest | Speedup |\n");

    out.push_str("| --- |");
    for _ in order {
        out.push_str(" ---: |");
    }
    out.push_str(" :---: | ---: |\n");

    for row in rows {
        out.push_str(&format!("| {} |", row.description));
        for library in order {
            out.push_str(&format!(" {} |", row.cell(library)));
        }
        let fastest = row.fastest().map(String::as_str).unwrap_or("-");
        let speedup = row
            .speedup_over_next()
            .map(|ratio| format!("{ratio:.2}x"))
            .unwrap_or_else(|| "-".into());
        out.push_str(&format!(" {fastest} | {speedup} |\n"));
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

    let mut all_libraries: BTreeSet<String> = BTreeSet::new();
    let rows: Vec<Row> = SCENARIOS
        .iter()
        .map(|(id, description)| {
            let values = discover(id);
            for library in values.keys() {
                all_libraries.insert(library.clone());
            }
            Row {
                description,
                values,
            }
        })
        .collect();

    if all_libraries.is_empty() {
        eprintln!(
            "Criterion results exist but no matching scenarios were found. Run `just bench`."
        );
        std::process::exit(1);
    }

    let order = order_libraries(&all_libraries);
    print_terminal_table(&order, &rows);

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let timestamp = utc_timestamp(secs);
    let markdown = markdown_report(&order, &rows, &timestamp);

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
