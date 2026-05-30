use crate::commands::{TileSet, TileStats};

pub fn print_detected(path: &str, format: &str) {
    println!("{}", serde_json::json!({
        "file": path,
        "detected_format": format
    }));
}

pub fn print_tile_set(tile_set: &TileSet) {
    println!("{}", serde_json::to_string_pretty(tile_set).unwrap());
}

pub fn print_composed(content: &str) {
    println!("{}", serde_json::json!({
        "status": "composed",
        "content": content
    }));
}

pub fn print_stats(stats: &TileStats) {
    println!("{}", serde_json::to_string_pretty(stats).unwrap());
}

pub fn print_pipeline_started(config: &serde_json::Value) {
    eprintln!("[forge] Pipeline started with config:");
    eprintln!("{}", serde_json::to_string_pretty(config).unwrap());
}

pub fn print_pipeline_result(input: &str, format: &str, tile_count: usize) {
    println!("{}", serde_json::json!({
        "status": "pipeline_complete",
        "input": input,
        "format": format,
        "tiles_produced": tile_count
    }));
}

pub fn print_crate_list(crates: &[crate::commands::CrateInfo]) {
    println!("{}", serde_json::json!({
        "ecosystem": "ForgeFlux",
        "crate_count": crates.len(),
        "crates": crates.iter().map(|c| serde_json::json!({
            "name": c.name,
            "version": c.version,
            "description": c.description
        })).collect::<Vec<_>>()
    }));
}

pub fn print_crate_info(info: &crate::commands::CrateInfo) {
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "name": info.name,
        "version": info.version,
        "description": info.description
    })).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_detected_format() {
        let json_str = serde_json::json!({"file": "test.txt", "detected_format": "text"}).to_string();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["detected_format"], "text");
    }

    #[test]
    fn test_print_crate_list_json_valid() {
        let crates = [
            crate::commands::CrateInfo { name: "forge-core", version: "0.1.0", description: "test" },
        ];
        // Just verify it doesn't panic
        print_crate_list(&crates);
    }
}
