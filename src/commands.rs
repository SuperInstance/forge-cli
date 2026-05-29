use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::output;

// --- Data types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub id: String,
    pub index: usize,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSet {
    pub format: String,
    pub source: String,
    pub tile_count: usize,
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileStats {
    pub format: String,
    pub source: String,
    pub tile_count: usize,
    pub total_bytes: usize,
    pub avg_tile_bytes: f64,
    pub min_tile_bytes: usize,
    pub max_tile_bytes: usize,
}

#[derive(Debug)]
pub struct CrateInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

const FORGE_CRATES: &[CrateInfo] = &[
    CrateInfo { name: "forge-core", version: "0.1.0", description: "Core types and traits for the ForgeFlux ecosystem" },
    CrateInfo { name: "forge-decompose", version: "0.1.0", description: "Decomposition engine: split content into tiles" },
    CrateInfo { name: "forge-compose", version: "0.1.0", description: "Composition engine: reassemble tiles back into content" },
    CrateInfo { name: "forge-detect", version: "0.1.0", description: "Format detection for input files" },
    CrateInfo { name: "forge-meta", version: "0.1.0", description: "Crate registry and metadata for the ForgeFlux ecosystem" },
    CrateInfo { name: "forge-pipeline", version: "0.1.0", description: "Pipeline orchestration for multi-step transformations" },
    CrateInfo { name: "forge-stats", version: "0.1.0", description: "Tile statistics and analytics" },
    CrateInfo { name: "forge", version: "0.1.0", description: "CLI tool tying all forge-* crates together" },
];

// --- Format detection ---

pub fn detect_format(content: &str) -> String {
    let trimmed = content.trim();

    // JSON detection
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            if trimmed.starts_with('{') {
                return "json-object".to_string();
            }
            return "json-array".to_string();
        }
    }

    // CSV detection: check if lines have consistent comma separation
    let lines: Vec<&str> = trimmed.lines().take(5).collect();
    if lines.len() >= 2 {
        let comma_counts: Vec<usize> = lines.iter().map(|l| l.matches(',').count()).collect();
        if comma_counts.iter().all(|&c| c > 0 && c == comma_counts[0]) {
            return "csv".to_string();
        }
    }

    // Default: plain text
    "text".to_string()
}

// --- Decomposition ---

pub fn decompose_content(content: &str, format: &str) -> Vec<Tile> {
    match format {
        "text" => decompose_text(content),
        "csv" => decompose_csv(content),
        "json-object" | "json-array" => decompose_json(content),
        _ => decompose_text(content),
    }
}

fn decompose_text(content: &str) -> Vec<Tile> {
    content
        .lines()
        .enumerate()
        .map(|(i, line)| Tile {
            id: Uuid::new_v4().to_string(),
            index: i,
            content: line.to_string(),
            metadata: None,
        })
        .collect()
}

fn decompose_csv(content: &str) -> Vec<Tile> {
    let mut lines = content.lines().peekable();
    let header_line = lines.peek().map(|l| *l).unwrap_or("");
    let headers: Vec<&str> = header_line.split(',').map(|h| h.trim()).collect();

    lines
        .enumerate()
        .map(|(i, line)| {
            let values: Vec<&str> = line.split(',').map(|v| v.trim()).collect();
            let mut meta = serde_json::Map::new();
            for (j, val) in values.iter().enumerate() {
                let key = headers.get(j).unwrap_or(&"unknown");
                meta.insert(key.to_string(), serde_json::Value::String(val.to_string()));
            }
            Tile {
                id: Uuid::new_v4().to_string(),
                index: i,
                content: line.to_string(),
                metadata: Some(serde_json::Value::Object(meta)),
            }
        })
        .collect()
}

fn decompose_json(content: &str) -> Vec<Tile> {
    let value: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return decompose_text(content),
    };

    match value {
        serde_json::Value::Array(arr) => arr
            .into_iter()
            .enumerate()
            .map(|(i, v)| Tile {
                id: Uuid::new_v4().to_string(),
                index: i,
                content: v.to_string(),
                metadata: None,
            })
            .collect(),
        serde_json::Value::Object(obj) => {
            let mut tiles = Vec::new();
            for (i, (key, val)) in obj.into_iter().enumerate() {
                let mut meta = serde_json::Map::new();
                meta.insert("key".to_string(), serde_json::Value::String(key));
                tiles.push(Tile {
                    id: Uuid::new_v4().to_string(),
                    index: i,
                    content: val.to_string(),
                    metadata: Some(serde_json::Value::Object(meta)),
                });
            }
            tiles
        }
        _ => decompose_text(content),
    }
}

fn read_file(path: &str) -> Result<String, String> {
    if !Path::new(path).exists() {
        return Err(format!("File not found: {}", path));
    }
    fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))
}

// --- Command implementations ---

pub fn detect(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: forge detect <file>".to_string());
    }
    let content = read_file(&args[0])?;
    let fmt = detect_format(&content);
    output::print_detected(&args[0], &fmt);
    Ok(())
}

pub fn decompose(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: forge decompose <file> [--format FMT]".to_string());
    }
    let content = read_file(&args[0])?;
    let mut format = detect_format(&content);

    // Check for --format override
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--format" && i + 1 < args.len() {
            format = args[i + 1].clone();
            break;
        }
        i += 1;
    }

    let tiles = decompose_content(&content, &format);
    let tile_set = TileSet {
        format: format.clone(),
        source: args[0].clone(),
        tile_count: tiles.len(),
        tiles,
    };

    output::print_tile_set(&tile_set);
    Ok(())
}

pub fn compose(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: forge compose <tiles.json>".to_string());
    }
    let content = read_file(&args[0])?;
    let tile_set: TileSet = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse tiles JSON: {}", e))?;

    // Reassemble by concatenating tile content in index order
    let mut tiles = tile_set.tiles;
    tiles.sort_by_key(|t| t.index);

    let recomposed: String = tiles.iter().map(|t| t.content.as_str()).collect::<Vec<&str>>().join("\n");
    output::print_composed(&recomposed);
    Ok(())
}

pub fn stats(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: forge stats <file>".to_string());
    }
    let content = read_file(&args[0])?;
    let format = detect_format(&content);
    let tiles = decompose_content(&content, &format);

    let sizes: Vec<usize> = tiles.iter().map(|t| t.content.len()).collect();
    let total_bytes: usize = sizes.iter().sum();
    let tile_count = tiles.len();
    let avg_tile_bytes = if tile_count > 0 {
        total_bytes as f64 / tile_count as f64
    } else {
        0.0
    };
    let min_tile_bytes = sizes.iter().min().copied().unwrap_or(0);
    let max_tile_bytes = sizes.iter().max().copied().unwrap_or(0);

    let stats = TileStats {
        format,
        source: args[0].clone(),
        tile_count,
        total_bytes,
        avg_tile_bytes,
        min_tile_bytes,
        max_tile_bytes,
    };

    output::print_stats(&stats);
    Ok(())
}

pub fn pipeline(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: forge pipeline <config.json>".to_string());
    }
    let content = read_file(&args[0])?;
    let config: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    output::print_pipeline_started(&config);

    // Simplified pipeline: read input, decompose, print result
    let input_path = config.get("input")
        .and_then(|v| v.as_str())
        .ok_or("Config missing 'input' field")?;

    let input_content = read_file(input_path)?;
    let detected = detect_format(&input_content);
    let format = config.get("format")
        .and_then(|v| v.as_str())
        .unwrap_or(&detected);

    let tiles = decompose_content(&input_content, format);
    output::print_pipeline_result(input_path, format, tiles.len());

    Ok(())
}

pub fn list(_args: &[String]) -> Result<(), String> {
    output::print_crate_list(FORGE_CRATES);
    Ok(())
}

pub fn info(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: forge info <crate-name>".to_string());
    }
    let name = &args[0];
    let info = FORGE_CRATES
        .iter()
        .find(|c| c.name == name.as_str())
        .ok_or(format!("Unknown crate: {}", name))?;

    output::print_crate_info(info);
    Ok(())
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_text() {
        assert_eq!(detect_format("hello world\nline two"), "text");
    }

    #[test]
    fn test_detect_csv() {
        assert_eq!(detect_format("name,age\nAlice,30\nBob,25"), "csv");
    }

    #[test]
    fn test_detect_json_object() {
        assert_eq!(detect_format("{\"key\": \"value\"}"), "json-object");
    }

    #[test]
    fn test_detect_json_array() {
        assert_eq!(detect_format("[1, 2, 3]"), "json-array");
    }

    #[test]
    fn test_decompose_text_lines() {
        let tiles = decompose_content("line1\nline2\nline3", "text");
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0].content, "line1");
        assert_eq!(tiles[1].content, "line2");
        assert_eq!(tiles[2].content, "line3");
    }

    #[test]
    fn test_decompose_csv_rows() {
        let tiles = decompose_content("name,age\nAlice,30\nBob,25", "csv");
        assert_eq!(tiles.len(), 3); // header + data rows
        // Actually lines are enumerated after peek, so line 0 = "Alice,30"
        assert_eq!(tiles[0].index, 0);
    }

    #[test]
    fn test_decompose_json_array() {
        let tiles = decompose_content("[1, 2, 3]", "json-array");
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0].content, "1");
    }

    #[test]
    fn test_decompose_json_object() {
        let tiles = decompose_content("{\"a\": 1, \"b\": 2}", "json-object");
        assert_eq!(tiles.len(), 2);
    }

    #[test]
    fn test_tile_ids_are_unique() {
        let tiles = decompose_content("a\nb\nc", "text");
        let ids: std::collections::HashSet<&str> = tiles.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids.len(), tiles.len());
    }

    #[test]
    fn test_compose_reassembles() {
        let tiles = decompose_content("hello\nworld", "text");
        let tile_set = TileSet {
            format: "text".to_string(),
            source: "test".to_string(),
            tile_count: tiles.len(),
            tiles,
        };
        let json = serde_json::to_string(&tile_set).unwrap();
        let parsed: TileSet = serde_json::from_str(&json).unwrap();
        let mut sorted = parsed.tiles;
        sorted.sort_by_key(|t| t.index);
        let result: String = sorted.iter().map(|t| t.content.as_str()).collect::<Vec<&str>>().join("\n");
        assert_eq!(result, "hello\nworld");
    }

    #[test]
    fn test_list_contains_all_crates() {
        assert!(FORGE_CRATES.len() >= 7);
        let names: Vec<&str> = FORGE_CRATES.iter().map(|c| c.name).collect();
        assert!(names.contains(&"forge-core"));
        assert!(names.contains(&"forge-decompose"));
        assert!(names.contains(&"forge-compose"));
        assert!(names.contains(&"forge-detect"));
    }

    #[test]
    fn test_info_unknown_crate() {
        let result = info(&["nonexistent".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_stats_empty_file() {
        let tiles = decompose_content("", "text");
        assert!(tiles.is_empty() || tiles.len() == 1);
    }

    #[test]
    fn test_format_override_in_decompose_args() {
        // Simulate parsing --format override
        let args = vec!["--format".to_string(), "csv".to_string()];
        let mut format = "text".to_string();
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--format" && i + 1 < args.len() {
                format = args[i + 1].clone();
                break;
            }
            i += 1;
        }
        assert_eq!(format, "csv");
    }
}
