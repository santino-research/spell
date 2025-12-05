// ─────────────────────────────────────────────────────────────────────────────
// SPELL
// Copyright (c) 2025 Santino Research. MIT License.
// ─────────────────────────────────────────────────────────────────────────────

//! SPELL command-line interface.

mod core;

use clap::Parser;
use std::fs;
use std::process;

#[derive(Parser)]
#[command(name = "spell")]
#[command(about = "SPELL - Dataflow programming for LLMs")]
struct Cli {
    /// SPELL program file (.json)
    file: String,
}

fn main() {
    let cli: Cli = Cli::parse();

    // Banner
    eprintln!("╔═══════════════════════════════════════╗");
    eprintln!("║  SPELL v0.1 (pre-alpha)               ║");
    eprintln!("║  Santino Research                     ║");
    eprintln!("╚═══════════════════════════════════════╝");
    eprintln!();

    let content: String = match fs::read_to_string(&cli.file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let graph: core::schema::Graph = match serde_json::from_str(&content) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let mut engine: core::engine::Engine = core::engine::Engine::new(graph);
    engine.run();
}
