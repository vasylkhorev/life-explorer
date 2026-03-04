use std::collections::{HashSet, HashMap};
use std::fs::OpenOptions;
use std::io::{Write, BufWriter};
use std::path::Path;
use rayon::prelude::*;
use rand::Rng;
use indicatif::{ProgressBar, ParallelProgressIterator};

use crate::rule::HalfLifeRule;
use crate::grid::Grid2D;
use crate::analysis::{analyze_pattern, PatternResult};

fn format_rule_for_key(rule: &HalfLifeRule) -> String {
    let b_min = HalfLifeRule::format_val(rule.b_min);
    let b_max = HalfLifeRule::format_val(rule.b_max);
    let s_min = HalfLifeRule::format_val(rule.s_min);
    let s_max = HalfLifeRule::format_val(rule.s_max);
    
    let mut s2_str = String::new();
    if let (Some(s2_min), Some(s2_max)) = (rule.s2_min, rule.s2_max) {
        s2_str = format!("_{}-{}", HalfLifeRule::format_val(s2_min), HalfLifeRule::format_val(s2_max));
    }
    
    format!("b{}-{}_s{}-{}{}", b_min, b_max, s_min, s_max, s2_str)
}

pub fn extract_patterns(rule: HalfLifeRule, num_patterns: usize, output_dir: &str, _threads: usize) {
    println!("Extracting patterns for rule {}...", rule);
    let pb = ProgressBar::new(num_patterns as u64);

    let results: Vec<_> = (0..num_patterns).into_par_iter()
        .progress_with(pb)
        .filter_map(|_| {
            let mut rng = rand::thread_rng();
            let gs = 32;
            let p_size = rng.gen_range(3..11);
            let density = rng.gen_range(0.15..0.45);

            let mut grid = Grid2D::new(gs, gs);
            let sy = (gs - p_size) / 2;
            let sx = (gs - p_size) / 2;

            for y in 0..p_size {
                for x in 0..p_size {
                    let r: f64 = rng.gen();
                    let val = if r < density / 2.0 {
                        2
                    } else if r < density {
                        1
                    } else {
                        0
                    };
                    grid.set(sx + x, sy + y, val);
                }
            }

            match analyze_pattern(&grid, &rule, 120) {
                PatternResult::Glider(sig, crop) => Some((sig, 0, crop)), // period 0 for gliders
                PatternResult::Oscillator { period, signature, best_phase } => Some((signature, period, best_phase)),
                _ => None,
            }
        })
        .collect();

    let mut gliders = Vec::new();
    let mut oscillators: HashMap<usize, Vec<Grid2D>> = HashMap::new();
    let mut seen_gliders = HashSet::new();
    let mut seen_oscs = HashSet::new();

    for (sig, period, crop) in results {
        if period == 0 {
            if seen_gliders.insert(sig) {
                gliders.push(crop);
            }
        } else {
            if seen_oscs.insert(sig) {
                oscillators.entry(period).or_default().push(crop);
            }
        }
    }

    println!("\nExtracted {} unique gliders.", gliders.len());
    let total_oscs: usize = oscillators.values().map(|v| v.len()).sum();
    println!("Extracted {} unique oscillators.", total_oscs);

    let path = Path::new(output_dir);
    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }

    let mut output_str = String::from("// Auto-generated patterns as RLE\nconst patterns = {\n");
    let rule_key = format_rule_for_key(&rule);
    output_str.push_str(&format!("  '{}': {{\n", rule_key));

    for (i, g) in gliders.iter().enumerate() {
        let name = format!("Glider {}", i + 1);
        let desc = format!("Stable glider found");
        let rle_body = g.to_rle();
        let full_rle = format!("x = {}, y = {}, rule = FuzzyLife/3\\n{}", g.width, g.height, rle_body);
        output_str.push_str(&format!("    '{}': {{\n", name));
        output_str.push_str(&format!("      rle: '{}',\n", full_rle));
        output_str.push_str(&format!("      description: '{}',\n", desc));
        output_str.push_str("    },\n");
    }

    let mut periods: Vec<_> = oscillators.keys().cloned().collect();
    periods.sort();

    for p in periods {
        for (i, o) in oscillators[&p].iter().enumerate() {
            let name = if p == 1 {
                format!("Still Life {}", i + 1)
            } else {
                format!("Oscillator P{} {}", p, i + 1)
            };

            let desc = format!("Period {} oscillator", p);
            let rle_body = o.to_rle();
            let full_rle = format!("x = {}, y = {}, rule = FuzzyLife/3\\n{}", o.width, o.height, rle_body);
            output_str.push_str(&format!("    '{}': {{\n", name));
            output_str.push_str(&format!("      rle: '{}',\n", full_rle));
            output_str.push_str(&format!("      description: '{}',\n", desc));
            output_str.push_str("    },\n");
        }
    }
    
    output_str.push_str("  },\n};\nexport default patterns;\n");

    let fpath = path.join(format!("{}_patterns.js", rule_key.replace(',', "_")));
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&fpath)
        .unwrap();
    let mut writer = BufWriter::new(file);
    writer.write_all(output_str.as_bytes()).unwrap();
    
    println!("All patterns exported to RLE JS file at {}", fpath.display());
}
