use cliff_watch_core::git::open_repository;
use std::path::Path;
use std::process;

/// Handle the report command - Generate audit report
pub fn handle_report(limit: usize, format: String) {
    let repo = match open_repository(Path::new(".")) {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("❌ Error opening repository: {}", e);
            process::exit(1);
        }
    };

    match cliff_watch_core::git::get_governance_history(&repo, limit) {
        Ok(entries) => {
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&entries)
                    .expect("Failed to serialize governance entries to JSON"));
                return;
            }

            let total_commits = entries.len();
            let total_score: f64 = entries.iter().map(|e| e.score).sum();
            let avg_score = if total_commits > 0 { total_score / total_commits as f64 } else { 0.0 };

            let mut authors = std::collections::HashMap::new();
            for e in &entries {
                *authors.entry(e.author.clone()).or_insert(0.0) += e.score;
            }

            if format == "md" {
                 println!("# Governance Audit Report");
                 println!("- **Analyzed Commits:** {}", total_commits);
                 println!("- **Total Energy:** {:.2}", total_score);
                 println!("- **Average Entropy:** {:.2}", avg_score);
                 println!("\n## Top Contributors (by Energy)");
                 println!("| Author | Energy |");
                 println!("|--------|--------|");
                 for (author, score) in &authors {
                     println!("| {} | {:.2} |", author, score);
                 }
            } else {
                println!("📊 Governance Report (Last {} commits)", limit);
                println!("--------------------------------");
                println!("  Commits Analyzed: {}", total_commits);
                println!("  Total Energy:     {:.2}", total_score);
                println!("  Average Score:    {:.2}", avg_score);
                println!("\n🏆 Top Contributors:");
                for (author, score) in authors {
                    println!("  - {}: {:.2}", author, score);
                }
            }
        }
        Err(e) => {
             eprintln!("❌ Failed to generate report: {}", e);
             process::exit(1);
        }
    }
}
