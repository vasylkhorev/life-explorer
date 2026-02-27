use std::error::Error;
use std::fs::File;
use std::io::Write;
use crate::explorer_2d::RuleStats;
use crate::explorer_1d::Rule1DStats;

pub fn write_2d_results_csv(results: &[RuleStats], filepath: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(filepath)?;
    
    // Header
    wtr.write_record(&[
        "Rule", "B_min", "B_max", "S_min", "S_max", 
        "Died", "Exploded", "Chaos", "Unique_Gliders", "Unique_Oscillators_Total",
        "P1_Count", "P2_Count", "P3_Count", "P4_Count", "P5_Count",
        "P6_Count", "P7_Count", "P8_Count", "P9_Count", "P10_Count"
    ])?;
    
    for r in results {
        let os_total: usize = r.oscillators.values().map(|s| s.len()).sum();
        
        let get_p = |p: usize| r.oscillators.get(&p).map(|s| s.len()).unwrap_or(0).to_string();
        
        wtr.write_record(&[
            r.rule.to_string(),
            r.rule.b_min.to_string(),
            r.rule.b_max.to_string(),
            r.rule.s_min.to_string(),
            r.rule.s_max.to_string(),
            r.dead.to_string(),
            r.explode.to_string(),
            r.chaos.to_string(),
            r.gliders.len().to_string(),
            os_total.to_string(),
            get_p(1), get_p(2), get_p(3), get_p(4), get_p(5),
            get_p(6), get_p(7), get_p(8), get_p(9), get_p(10),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

pub fn write_1d_results_csv(results: &[Rule1DStats], filepath: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(filepath)?;
    
    wtr.write_record(&[
        "Weights", "Rule", "w1", "w2", "w3", "B_min", "B_max", "S_min", "S_max",
        "Dead", "Expand", "Chaos", "Spaceship", "Oscillator"
    ])?;
    
    for r in results {
        let weights_str = format!("{}_{}_{}", r.weights.0, r.weights.1, r.weights.2);
        let rule_str = format!("B{}-{}/S{}-{}", r.b_min, r.b_max, r.s_min, r.s_max);
        
        wtr.write_record(&[
            weights_str,
            rule_str,
            r.weights.0.to_string(),
            r.weights.1.to_string(),
            r.weights.2.to_string(),
            r.b_min.to_string(),
            r.b_max.to_string(),
            r.s_min.to_string(),
            r.s_max.to_string(),
            r.dead.to_string(),
            r.expand.to_string(),
            r.chaos.to_string(),
            r.spaceship.to_string(),
            r.oscillator.to_string(),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

pub fn generate_html_table(csv_filepath: &str, html_filepath: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(csv_filepath)?;
    let headers = rdr.headers()?.clone();
    
    let mut html = String::from("<!DOCTYPE html>
<html lang=\"sk\">
<head>
    <meta charset=\"UTF-8\">
    <title>Výsledky prieskumu pravidiel</title>
    <script src=\"https://code.jquery.com/jquery-3.7.0.min.js\"></script>
    <link rel=\"stylesheet\" href=\"https://cdn.datatables.net/1.13.6/css/jquery.dataTables.min.css\">
    <script src=\"https://cdn.datatables.net/1.13.6/js/jquery.dataTables.min.js\"></script>
    <style>
        body { font-family: 'Segoe UI', sans-serif; background-color: #0f172a; color: #e2e8f0; padding: 20px; }
        .container { max-width: 1400px; margin: 0 auto; background-color: #1e293b; padding: 30px; border-radius: 12px; }
        h1 { color: #38bdf8; }
        table.dataTable { color: #e2e8f0; }
        table.dataTable thead th { border-bottom: 2px solid #334155; color: #38bdf8; }
        table.dataTable tbody tr { background-color: transparent !important; }
        table.dataTable tbody tr:hover { background-color: #334155 !important; }
        .dataTables_wrapper .dataTables_length, .dataTables_wrapper .dataTables_filter, .dataTables_wrapper .dataTables_info, .dataTables_wrapper .dataTables_paginate { color: #94a3b8 !important; }
        .dataTables_wrapper .dataTables_paginate .paginate_button { color: #e2e8f0 !important; }
        .dataTables_wrapper .dataTables_paginate .paginate_button.current { background: #38bdf8 !important; color: #0f172a !important; }
    </style>
</head>
<body>
    <div class=\"container\">
        <h1>Výsledky prieskumu (Half-Life Rust)</h1>
        <table id=\"rulesTable\" class=\"display\" style=\"width:100%\">
            <thead>
                <tr>\n");

    for h in headers.iter() {
        html.push_str(&format!("                    <th>{}</th>\n", h.replace('_', " ")));
    }
    html.push_str("                </tr>\n            </thead>\n            <tbody>\n");

    for result in rdr.records() {
        let record = result?;
        html.push_str("                <tr>\n");
        for (i, val) in record.iter().enumerate() {
            let col_name = headers.get(i).unwrap_or("");
            if (col_name == "Unique_Gliders" || col_name == "Spaceship") && val.parse::<i32>().unwrap_or(0) > 0 {
                html.push_str(&format!("                    <td style=\"color: #4ade80; font-weight: bold;\">{}</td>\n", val));
            } else if (col_name == "Exploded" || col_name == "Expand") && val.parse::<i32>().unwrap_or(0) > 900 {
                html.push_str(&format!("                    <td style=\"color: #f87171;\">{}</td>\n", val));
            } else {
                html.push_str(&format!("                    <td>{}</td>\n", val));
            }
        }
        html.push_str("                </tr>\n");
    }

    html.push_str("            </tbody>
        </table>
    </div>
    <script>
        $(document).ready(function() {
            $('#rulesTable').DataTable({
                \"order\": [[ 8, \"desc\" ], [ 9, \"desc\" ]],
                \"pageLength\": 50
            });
        });
    </script>
</body>
</html>");

    let mut file = File::create(html_filepath)?;
    file.write_all(html.as_bytes())?;
    
    Ok(())
}
