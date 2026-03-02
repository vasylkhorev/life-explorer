# Half-Life Explorer

Half-Life Explorer is a high-performance Rust-based search tool for exploring and classifying rules and patterns in **Totalistic Cellular Automata** (specifically "Half-Life" or "Fuzzy Life" variants). It investigates how different birth (B) and survival (S) conditions affect grid dynamics, identifying stable patterns like oscillators and spaceships (gliders).

This project is a companion to the [Fuzzy Life](https://fuzzylife.netlify.app/) web application.

## 🔗 Project Links

- **Web Application:** [fuzzylife.netlify.app](https://fuzzylife.netlify.app/)
- **Core Engine Repository:** [github.com/vasylkhorev/fuzzy-life](https://github.com/vasylkhorev/fuzzy-life)

## 🚀 Key Features

- **2D & 1D Exploration:** Automated search across thousands of rule combinations.
- **Pattern Classification:** Automatically identifies:
  - **Dead:** Patterns that vanish.
  - **Explode:** Patterns that grow uncontrollably.
  - **Oscillators:** Stable repeating patterns (Still Lifes are Period 1).
  - **Spaceships/Gliders:** Moving stable patterns.
- **Advanced Rule Support:** Supports split survival intervals (e.g., `B4/S2-3,6-8`), allowing for complex life-like behaviors.
- **Normalized Output:** Rules are presented in a normalized format (values divided by 2) to match the standard Half-Life notation (e.g., `B1` instead of `B2`).
- **Data Export:** Results are exported to CSV and interactive HTML tables for easy analysis.
- **Pattern Extraction:** Extract discovered patterns directly into JavaScript format for use in the Fuzzy Life web app.

## 🛠️ Usage

Build the project using Cargo:

```bash
cargo build --release
```

### Run 2D Exploration
Explore all even-split rules (11,475 combinations) or the full set:

```bash
# Run with default 'all' mode (23,409 rules)
cargo run --release -- explore-2d --patterns 3000

# Run with even-split mode
cargo run --release -- explore-2d --mode even-split --patterns 3000
```

### Extract Patterns
Save gliders and oscillators for a specific rule to a directory:

```bash
cargo run --release -- extract --rule "B4/S6-10" --patterns 10000 --dir patterns_out
```

### Convert Results to HTML
Transform a CSV result file into a searchable, sortable HTML table:

```bash
cargo run --release -- csv-to-html --csv explore_2d.csv --html results.html
```

## 📊 Combinatorics

The explorer is optimized to handle large search spaces. For example, the `even-split` mode tests **11,475 unique rules** by combining 45 Birth intervals with 255 Survival combinations (including single and multi-interval survival sets).

---
© [Vasiľ Chorev](https://github.com/vasylkhorev)
