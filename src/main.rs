pub mod grid;
pub mod rule;
pub mod components;
pub mod glider;
pub mod analysis;
pub mod explorer_2d;
pub mod explorer_1d;
pub mod extractor;
pub mod output;

use clap::{Parser, Subcommand};
use crate::rule::HalfLifeRule;
use std::io::Write;

#[derive(Parser)]
#[command(name = "halflife-explorer")]
#[command(about = "Half-Life Cellular Automata Explorer", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, default_value_t = 12)]
    threads: usize,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the 2D rule explorer
    #[command(name = "explore-2d")]
    Explore2d {
        /// Number of patterns to test per rule
        #[arg(long, default_value_t = 3000)]
        patterns: usize,
        
        /// Rule type: 'all', 'comprehensive', 'even', or 'even-split'
        #[arg(long, default_value = "all")]
        mode: String,
        
        /// Output CSV file
        #[arg(short, long, default_value = "explore_2d.csv")]
        out: String,
    },
    
    /// Run the 1D weighted rule explorer
    #[command(name = "explore-1d")]
    Explore1d {
        /// Number of patterns to test per rule
        #[arg(long, default_value_t = 500)]
        patterns: usize,
        
        /// Output CSV file
        #[arg(short, long, default_value = "explore_1d.csv")]
        out: String,
    },
    
    /// Extract gliders and oscillators for a specific rule to JSON files
    Extract {
        /// The rule to extract, e.g. "B4-4/S6-10"
        #[arg(long)]
        rule: String,
        
        /// Number of patterns to simulate
        #[arg(long, default_value_t = 10000)]
        patterns: usize,
        
        /// Output directory for JSON files
        #[arg(short, long, default_value = "patterns_out")]
        dir: String,
    },
    
    /// Convert CSV output to an interactive HTML table
    #[command(name = "csv-to-html")]
    CsvToHtml {
        #[arg(long)]
        csv: String,
        
        #[arg(long)]
        html: String,
    }
}

fn parse_rule_str(mut s: &str) -> HalfLifeRule {
    // Basic parser for "B4-6/S1-3" or "B4/S4-7"
    if s.starts_with('B') {
        s = &s[1..];
    }
    
    let parse_val = |v: &str| -> i32 {
        (v.parse::<f32>().unwrap() * 2.0).round() as i32
    };
    
    let parts: Vec<&str> = s.split("/S").collect();
    if parts.len() != 2 {
        panic!("Invalid rule format. Use Bmin-max/Smin-max");
    }
    
    let b_parts: Vec<&str> = parts[0].split('-').collect();
    let (b_min, b_max) = if b_parts.len() == 2 {
        (parse_val(b_parts[0]), parse_val(b_parts[1]))
    } else {
        (parse_val(b_parts[0]), parse_val(b_parts[0]))
    };
    
    let s_parts_all: Vec<&str> = parts[1].split(',').collect();
    let s_parts: Vec<&str> = s_parts_all[0].split('-').collect();
    let (s_min, s_max) = if s_parts.len() == 2 {
        (parse_val(s_parts[0]), parse_val(s_parts[1]))
    } else {
        (parse_val(s_parts[0]), parse_val(s_parts[0]))
    };
    
    if s_parts_all.len() > 1 {
        let s2_parts: Vec<&str> = s_parts_all[1].split('-').collect();
        let (s2_min, s2_max) = if s2_parts.len() == 2 {
            (parse_val(s2_parts[0]), parse_val(s2_parts[1]))
        } else {
            (parse_val(s2_parts[0]), parse_val(s2_parts[0]))
        };
        HalfLifeRule::new_with_s2(b_min, b_max, s_min, s_max, s2_min, s2_max)
    } else {
        HalfLifeRule::new(b_min, b_max, s_min, s_max)
    }
}

fn main() {
    let cli = Cli::parse();
    
    rayon::ThreadPoolBuilder::new().num_threads(cli.threads).build_global().unwrap();
    
    match cli.command {
        Commands::Explore2d { patterns, mode, out } => {
            let mut rules = Vec::new();
            
            if mode == "even-split" {
                let evens = [0, 2, 4, 6, 8, 10, 12, 14, 16];
                for &b_min in &evens {
                    for &b_max in &evens {
                        if b_min > b_max { continue; }
                        for i in 0..evens.len() {
                            for j in i..evens.len() {
                                let s1_min = evens[i];
                                let s1_max = evens[j];
                                rules.push(HalfLifeRule::new(b_min, b_max, s1_min, s1_max));
                                
                                // Second interval must start after first interval ends
                                for k in (j+2)..evens.len() {
                                    for l in k..evens.len() {
                                        let s2_min = evens[k];
                                        let s2_max = evens[l];
                                        rules.push(HalfLifeRule::new_with_s2(b_min, b_max, s1_min, s1_max, s2_min, s2_max));
                                    }
                                }
                            }
                        }
                    }
                }
            } else if mode == "even" {
                let evens = [0, 2, 4, 6, 8, 10, 12, 14, 16];
                for &b_min in &evens {
                    for &b_max in &evens {
                        if b_min > b_max { continue; }
                        for &s_min in &evens {
                            for &s_max in &evens {
                                if s_min > s_max { continue; }
                                rules.push(HalfLifeRule::new(b_min, b_max, s_min, s_max));
                            }
                        }
                    }
                }
            } else if mode == "all" {
                for b_min in 0..=16 {
                    for b_max in b_min..=16 {
                        for s_min in 0..=16 {
                            for s_max in s_min..=16 {
                                rules.push(HalfLifeRule::new(b_min, b_max, s_min, s_max));
                            }
                        }
                    }
                }
            } else { // comprehensive
                for b_min in 3..7 {
                    for b_max in b_min..std::cmp::min(b_min + 4, 13) {
                        for s_min in 1..5 {
                            for s_max in s_min..std::cmp::min(s_min + 4, 9) {
                                rules.push(HalfLifeRule::new(b_min, b_max, s_min, s_max));
                            }
                        }
                    }
                }
            }
            
            
            let stats = explorer_2d::explore_2d(rules, patterns, cli.threads);
            
            println!("Post-processing: Sorting results...");
            let _ = std::io::stdout().flush();
            // Results are already sorted in explore_2d
            
            println!("Post-processing: Writing CSV and HTML output...");
            let _ = std::io::stdout().flush();
            output::write_2d_results_csv(&stats, &out).unwrap();
            
            let html_out = out.replace(".csv", ".html");
            output::generate_html_table(&out, &html_out).unwrap();
            println!("Done! Results saved to {} and {}", out, html_out);
        },
        
        Commands::Explore1d { patterns, out } => {
            let stats = explorer_1d::explore_1d(patterns, cli.threads);
            output::write_1d_results_csv(&stats, &out).unwrap();
            
            let html_out = out.replace(".csv", ".html");
            output::generate_html_table(&out, &html_out).unwrap();
            println!("Results saved to {} and {}", out, html_out);
        },
        
        Commands::Extract { rule, patterns, dir } => {
            let parsed_rule = parse_rule_str(&rule);
            extractor::extract_patterns(parsed_rule, patterns, &dir, cli.threads);
        },
        
        Commands::CsvToHtml { csv, html } => {
            output::generate_html_table(&csv, &html).unwrap();
            println!("HTML generated at {}", html);
        }
    }
}
