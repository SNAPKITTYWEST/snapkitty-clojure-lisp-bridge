//! bifrost CLI — seal and verify chain events from the command line.
//!
//! ```
//! bifrost seal   --chain ./chain --type cap_transfer \
//!                --from <cid> --to <cid> --cap <cid> --policy <cid>
//!
//! bifrost verify --chain ./chain --head <cid>
//!
//! bifrost head   --chain ./chain
//! ```

use std::process;
use bifrost::event::Cid;
use bifrost::seal::Bifrost;
use bifrost::verify::verify_chain;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: bifrost <seal|verify|head> [options]");
        process::exit(1);
    }

    match args[1].as_str() {
        "seal"   => cmd_seal(&args[2..]),
        "verify" => cmd_verify(&args[2..]),
        "head"   => cmd_head(&args[2..]),
        cmd      => { eprintln!("unknown command: {cmd}"); process::exit(1); }
    }
}

fn parse_flags(args: &[String]) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut i = 0;
    while i + 1 < args.len() {
        if args[i].starts_with("--") {
            map.insert(args[i][2..].to_string(), args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }
    map
}

fn cmd_seal(args: &[String]) {
    let flags = parse_flags(args);
    let chain_path = flags.get("chain").map(|s| s.as_str()).unwrap_or("./chain");

    let mut bifrost = Bifrost::open(chain_path).unwrap_or_else(|e| {
        eprintln!("bifrost open: {e}"); process::exit(1);
    });

    // Auto-genesis if chain is empty.
    if bifrost.dag.head().unwrap().is_none() {
        let g = bifrost.seal_genesis().unwrap_or_else(|e| {
            eprintln!("genesis: {e}"); process::exit(1);
        });
        println!("genesis {g}");
    }

    let event_type = flags.get("type").map(|s| s.as_str()).unwrap_or("cap_transfer");

    let cid = match event_type {
        "cap_transfer" => {
            let from       = cid_flag(&flags, "from");
            let to         = cid_flag(&flags, "to");
            let cap_hash   = cid_flag(&flags, "cap");
            let policy_cid = cid_flag(&flags, "policy");

            // Seal policy blob if a path was given as the policy flag.
            let policy_cid = if flags.get("policy-file").is_some() {
                let bytes = std::fs::read(flags["policy-file"].as_str()).unwrap();
                bifrost.worm.seal_blob(&bytes, None).unwrap()
            } else {
                policy_cid
            };

            bifrost.seal_cap_transfer(from, to, cap_hash, policy_cid).unwrap_or_else(|e| {
                eprintln!("seal cap_transfer: {e}"); process::exit(1);
            })
        }
        "jit_compile" => {
            let soulir_cid   = cid_flag(&flags, "soulir");
            let wasm_cid     = cid_flag(&flags, "wasm");
            let opt_level    = flags.get("opt-level").and_then(|s| s.parse().ok()).unwrap_or(1u8);
            let gas_estimate = flags.get("gas").and_then(|s| s.parse().ok()).unwrap_or(0u64);
            bifrost.seal_jit_compile(soulir_cid, wasm_cid, opt_level, gas_estimate).unwrap_or_else(|e| {
                eprintln!("seal jit_compile: {e}"); process::exit(1);
            })
        }
        t => { eprintln!("unknown event type: {t}"); process::exit(1); }
    };

    println!("{cid}");
}

fn cmd_verify(args: &[String]) {
    let flags = parse_flags(args);
    let chain_path = flags.get("chain").map(|s| s.as_str()).unwrap_or("./chain");

    let bifrost = Bifrost::open(chain_path).unwrap_or_else(|e| {
        eprintln!("bifrost open: {e}"); process::exit(1);
    });

    let head = match flags.get("head") {
        Some(h) => Cid::from_hex(h).unwrap_or_else(|e| {
            eprintln!("invalid head CID: {e}"); process::exit(1);
        }),
        None => bifrost.dag.head().unwrap_or_else(|e| {
            eprintln!("dag head: {e}"); process::exit(1);
        }).unwrap_or_else(|| {
            eprintln!("chain is empty"); process::exit(1);
        }),
    };

    let report = verify_chain(&head, &bifrost.dag, &bifrost.worm).unwrap_or_else(|e| {
        eprintln!("verify: {e}"); process::exit(1);
    });

    println!("head:   {}", report.head);
    println!("height: {}", report.height);
    println!("events: {}", report.event_count);
    if report.ok {
        println!("status: OK");
    } else {
        println!("status: FAIL ({} errors)", report.errors.len());
        for e in &report.errors { println!("  - {e}"); }
        process::exit(2);
    }
}

fn cmd_head(args: &[String]) {
    let flags = parse_flags(args);
    let chain_path = flags.get("chain").map(|s| s.as_str()).unwrap_or("./chain");
    let bifrost = Bifrost::open(chain_path).unwrap_or_else(|e| {
        eprintln!("bifrost open: {e}"); process::exit(1);
    });
    match bifrost.dag.head().unwrap() {
        Some(cid) => println!("{cid}"),
        None      => { eprintln!("chain is empty"); process::exit(1); }
    }
}

fn cid_flag(flags: &std::collections::HashMap<String, String>, name: &str) -> Cid {
    let hex = flags.get(name).unwrap_or_else(|| {
        eprintln!("missing --{name}"); process::exit(1);
    });
    Cid::from_hex(hex).unwrap_or_else(|e| {
        eprintln!("invalid CID for --{name}: {e}"); process::exit(1);
    })
}
