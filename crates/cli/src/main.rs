use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};

use patchwaste_core::report::Report;
use patchwaste_core::types::Severity;
use patchwaste_core::{analyze_dir, AnalyzeOptions};

#[derive(Parser, Debug)]
#[command(
    name = "patchwaste",
    version,
    about = "SteamPipe patch efficiency gate (estimated)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Analyze {
        #[arg(long)]
        input: PathBuf,

        #[arg(long)]
        baseline: Option<PathBuf>,

        #[arg(long)]
        budget_ratio: Option<f64>,

        #[arg(long)]
        strict: bool,

        #[arg(long, default_value = "patchwaste-out")]
        out: PathBuf,
    },
}

struct Style {
    bold: &'static str,
    dim: &'static str,
    red: &'static str,
    green: &'static str,
    yellow: &'static str,
    orange: &'static str,
    reset: &'static str,
}

const COLOR: Style = Style {
    bold: "\x1b[1m",
    dim: "\x1b[2m",
    red: "\x1b[31m",
    green: "\x1b[32m",
    yellow: "\x1b[33m",
    orange: "\x1b[38;5;208m",
    reset: "\x1b[0m",
};

const PLAIN: Style = Style {
    bold: "",
    dim: "",
    red: "",
    green: "",
    yellow: "",
    orange: "",
    reset: "",
};

fn style() -> &'static Style {
    if std::env::var_os("NO_COLOR").is_some() {
        &PLAIN
    } else {
        &COLOR
    }
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();

    let res = match cli.cmd {
        Commands::Analyze {
            input,
            baseline,
            budget_ratio,
            strict,
            out,
        } => run_analyze(&input, baseline.as_deref(), budget_ratio, strict, &out),
    };

    match res {
        Ok(code) => code,
        Err(e) => {
            let s = style();
            eprintln!(
                "{}{red}error:{reset} {:#}",
                s.bold,
                e,
                red = s.red,
                reset = s.reset
            );
            std::process::ExitCode::from(1)
        }
    }
}

fn print_banner() {
    let s = style();
    eprintln!(
        "\n  {bold}patch{reset}{orange}|{reset}{dim}waste{reset}  {dim}steampipe efficiency gate{reset}\n",
        bold = s.bold,
        orange = s.orange,
        dim = s.dim,
        reset = s.reset,
    );
}

fn waste_color(ratio: f64) -> &'static str {
    let s = style();
    if ratio < 0.3 {
        s.green
    } else if ratio < 0.5 {
        s.yellow
    } else {
        s.red
    }
}

fn severity_color(sev: &Severity) -> &'static str {
    let s = style();
    match sev {
        Severity::High => s.red,
        Severity::Medium => s.yellow,
        Severity::Low => s.dim,
    }
}

fn commas(n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

fn print_report(report: &Report, out: &Path) {
    let s = style();
    let wc = waste_color(report.metrics.waste_ratio);

    eprintln!(
        "  {dim}new_bytes             {reset}{bold}{}{reset}",
        commas(report.metrics.new_bytes),
        dim = s.dim,
        bold = s.bold,
        reset = s.reset
    );
    eprintln!(
        "  {dim}changed_content_bytes {reset}{bold}{}{reset}",
        commas(report.metrics.changed_content_bytes),
        dim = s.dim,
        bold = s.bold,
        reset = s.reset
    );
    eprintln!(
        "  {dim}waste_ratio           {reset}{wc}{bold}{:.3}{reset}",
        report.metrics.waste_ratio,
        dim = s.dim,
        wc = wc,
        bold = s.bold,
        reset = s.reset
    );
    eprintln!(
        "  {dim}delta_efficiency      {reset}{bold}{:.3}{reset}",
        report.metrics.delta_efficiency,
        dim = s.dim,
        bold = s.bold,
        reset = s.reset
    );

    if !report.findings.is_empty() {
        eprintln!();
        for f in &report.findings {
            let sc = severity_color(&f.severity);
            eprintln!(
                "  {sc}{:?}{reset}  {}",
                f.severity,
                f.id,
                sc = sc,
                reset = s.reset
            );
        }
    }

    eprintln!();
    eprintln!(
        "  {dim}\u{2192} {}{reset}",
        out.join("report.json").display(),
        dim = s.dim,
        reset = s.reset
    );
    eprintln!(
        "  {dim}\u{2192} {}{reset}",
        out.join("report.md").display(),
        dim = s.dim,
        reset = s.reset
    );
    eprintln!();
}

fn run_analyze(
    input: &Path,
    baseline: Option<&Path>,
    budget_ratio: Option<f64>,
    strict: bool,
    out: &Path,
) -> anyhow::Result<std::process::ExitCode> {
    let s = style();

    print_banner();

    let opts = AnalyzeOptions {
        strict,
        budget_ratio,
        baseline_path: baseline.map(|p| p.to_path_buf()),
        ..AnalyzeOptions::default()
    };

    let mut report = analyze_dir(input, opts)?;
    report.inputs.input_path = input.display().to_string();

    std::fs::create_dir_all(out).with_context(|| format!("create out dir {}", out.display()))?;

    let json_path = out.join("report.json");
    let md_path = out.join("report.md");

    let json = serde_json::to_vec_pretty(&report).context("serialize report json")?;
    std::fs::write(&json_path, json).with_context(|| format!("write {}", json_path.display()))?;

    let md = report.to_markdown();
    std::fs::write(&md_path, md).with_context(|| format!("write {}", md_path.display()))?;

    // Machine-parseable line on stdout
    println!(
        "new_bytes={} changed_content_bytes={} waste_ratio={:.3}",
        report.metrics.new_bytes, report.metrics.changed_content_bytes, report.metrics.waste_ratio
    );

    // Human-readable output on stderr
    print_report(&report, out);

    let exit = match &report.budget {
        Some(b) if !b.pass => {
            eprintln!(
                "  {red}{bold}BUDGET FAILED{reset}  {dim}({:.2}x > {:.2}x budget){reset}",
                report
                    .baseline_comparison
                    .as_ref()
                    .map(|c| c.regression_ratio)
                    .unwrap_or(0.0),
                b.threshold_regression_ratio,
                red = s.red,
                bold = s.bold,
                dim = s.dim,
                reset = s.reset,
            );
            std::process::ExitCode::from(2)
        }
        _ => {
            eprintln!(
                "  {green}{bold}PASS{reset}",
                green = s.green,
                bold = s.bold,
                reset = s.reset
            );
            std::process::ExitCode::from(0)
        }
    };

    eprintln!();

    Ok(exit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn commas_formats_numbers() {
        assert_eq!(commas(0), "0");
        assert_eq!(commas(1000), "1,000");
        assert_eq!(commas(1234567), "1,234,567");
    }

    #[test]
    fn waste_color_thresholds() {
        assert_eq!(waste_color(0.1), style().green);
        assert_eq!(waste_color(0.4), style().yellow);
        assert_eq!(waste_color(0.8), style().red);
    }

    #[test]
    fn severity_color_thresholds() {
        assert_eq!(severity_color(&Severity::High), style().red);
        assert_eq!(severity_color(&Severity::Medium), style().yellow);
        assert_eq!(severity_color(&Severity::Low), style().dim);
    }

    #[test]
    #[serial]
    fn style_respects_no_color() {
        std::env::set_var("NO_COLOR", "1");
        assert_eq!(style().bold, "");
        std::env::remove_var("NO_COLOR");
        assert_ne!(style().bold, "");
    }
}
