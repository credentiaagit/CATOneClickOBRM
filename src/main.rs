/*!
 * OneClick BRM Rust Server - Session Management API
 * 
 * This application demonstrates key Rust concepts including:
 * - Ownership, borrowing, and lifetimes
 * - Async/await programming
 * - Error handling patterns
 * - Trait implementations and generics
 * - Memory safety with Arc<Mutex<T>>
 * - Serde serialization/deserialization
 * - Custom middleware implementation
 */

// === IMPORTS SECTION ===
// This demonstrates Rust's module system and explicit imports

// External crate imports using the `use` keyword
// Actix-web: Async web framework for Rust
use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware::Logger, dev::ServiceRequest, dev::ServiceResponse, Error};
use actix_web::dev::{forward_ready, Service, Transform};
use actix_cors::Cors;
use actix_multipart::Multipart;
use futures_util::TryStreamExt as _;
use std::io::{Write, Read, Cursor};
use regex::Regex;
use zip::{ZipArchive, ZipWriter, write::FileOptions};

// Serde: Serialization/deserialization framework
// The derive macros automatically implement traits for our structs
use serde::{Deserialize, Serialize};

// Standard library imports
use std::collections::HashMap;           // Hash map for key-value storage
use std::sync::{Arc, Mutex};            // Thread-safe reference counting and mutual exclusion
use chrono::{DateTime, Utc};            // Date/time handling with timezone support
use std::future::{Ready, ready};        // Future types for async programming
use std::pin::Pin;                      // Pin type for self-referential structures
// Removed duplicate import - already imported above
use std::fs::OpenOptions;               // File system operations
use log::{info, error};                 // Logging macros
use std::path::Path;                    // Path manipulation utilities

// BRM Module for Oracle BRM field management
mod brm;

// === CALLSTACK ANALYSIS STRUCTURES ===
// Data structures for analyzing Oracle BRM callstack traces

/// Represents a single entry in the callstack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStackEntry {
    pub timestamp: f64,
    pub level: usize,
    pub is_enter: bool,
    pub operation: String,
    pub return_code: String,
}

/// Statistics for operation performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStats {
    pub total_duration: f64,
    pub call_count: usize,
    pub min_duration: f64,
    pub max_duration: f64,
    pub individual_durations: Vec<f64>,
}

impl OperationStats {
    pub fn new() -> Self {
        Self {
            total_duration: 0.0,
            call_count: 0,
            min_duration: f64::MAX,
            max_duration: f64::MIN,
            individual_durations: Vec::new(),
        }
    }

    pub fn add_duration(&mut self, duration: f64) {
        self.total_duration += duration;
        self.call_count += 1;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
        self.individual_durations.push(duration);
    }

    pub fn average_duration(&self) -> f64 {
        if self.call_count > 0 {
            self.total_duration / self.call_count as f64
        } else {
            0.0
        }
    }
}

/// Main analyzer for callstack data
#[derive(Debug)]
pub struct CallStackAnalyzer {
    entries: Vec<CallStackEntry>,
    operation_stats: std::collections::HashMap<String, OperationStats>,
}

impl CallStackAnalyzer {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            operation_stats: std::collections::HashMap::new(),
        }
    }

    pub fn parse_callstack(&mut self, input: &str) -> Result<(), String> {
        let re = Regex::new(r"^\s*(\d+\.\d+)\s+(\.*)(\w+)\s+(\w+)\s+\(([^)]+)\)")
            .map_err(|e| format!("Regex compilation error: {}", e))?;

        for line in input.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Some(captures) = re.captures(line) {
                let timestamp = captures[1].parse::<f64>()
                    .map_err(|e| format!("Failed to parse timestamp: {}", e))?;
                
                let level = captures[2].len(); // Count the dots for nesting level
                let action = &captures[3];
                let operation = captures[4].to_string();
                let return_code = captures[5].to_string();

                let is_enter = action == "Enter";

                let entry = CallStackEntry {
                    timestamp,
                    level,
                    is_enter,
                    operation,
                    return_code,
                };

                self.entries.push(entry);
            } else {
                return Err(format!("Failed to parse line: {}", line));
            }
        }

        Ok(())
    }

    pub fn calculate_durations(&mut self) {
        let mut call_stack: Vec<(String, f64)> = Vec::new(); // (operation, start_time)

        for entry in &self.entries {
            if entry.is_enter {
                call_stack.push((entry.operation.clone(), entry.timestamp));
            } else {
                if let Some((op_name, start_time)) = call_stack.pop() {
                    if op_name == entry.operation {
                        let duration = entry.timestamp - start_time;
                        
                        let stats = self.operation_stats.entry(op_name.clone()).or_insert_with(OperationStats::new);
                        stats.add_duration(duration);
                    }
                }
            }
        }
    }

    pub fn find_longest_operations_by_total(&self, top_n: usize) -> Vec<(&String, &OperationStats)> {
        let mut sorted: Vec<_> = self.operation_stats.iter().collect();
        sorted.sort_by(|a, b| b.1.total_duration.partial_cmp(&a.1.total_duration).unwrap());
        sorted.into_iter().take(top_n).collect()
    }

    pub fn generate_html_report(&self, threshold_seconds: Option<f64>) -> String {
        let html_content = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Callstack Analysis Report</title>
    <style>
        {}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Callstack Analysis Report</h1>
            <p class="subtitle">Generated from {} entries</p>
        </header>
        
        <nav class="tabs">
            <button class="tab-button active" onclick="showTab('tree')">Call Tree</button>
            <button class="tab-button" onclick="showTab('stats')">Statistics</button>
            <button class="tab-button" onclick="showTab('top-total')">Top by Total</button>
            <button class="tab-button" onclick="showTab('top-avg')">Top by Average</button>
        </nav>

        <div id="tree" class="tab-content active">
            <h2>Interactive Call Tree</h2>
            <p style="color: #6c757d; margin-bottom: 15px; font-size: 0.9em;">
                📁 Click on operation nodes to expand/collapse their children • Duration badges show individual call times
                <br>⌨️ Keyboard shortcuts: <kbd>Ctrl+E</kbd> Expand All, <kbd>Ctrl+C</kbd> Collapse All
            </p>
            <div class="tree-container">
                {}
            </div>
        </div>

        <div id="stats" class="tab-content">
            <h2>Operation Statistics Summary</h2>
            {}
            <div class="table-container">
                {}
            </div>
        </div>

        <div id="top-total" class="tab-content">
            <h2>Top Operations by Total Duration</h2>
            <div class="ranking-container">
                {}
            </div>
        </div>

        <div id="top-avg" class="tab-content">
            <h2>Top Operations by Average Duration</h2>
            <div class="ranking-container">
                {}
            </div>
        </div>
    </div>

    <script>
        {}
    </script>
</body>
</html>"#, 
            self.generate_css(),
            self.entries.len(),
            self.generate_html_tree(threshold_seconds),
            self.generate_threshold_info(threshold_seconds),
            self.generate_html_stats_table(threshold_seconds),
            self.generate_html_top_total(),
            self.generate_html_top_average(),
            self.generate_javascript()
        );

        html_content
    }

    fn generate_css(&self) -> String {
        r#"
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            min-height: 100vh;
            padding: 20px;
        }

        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 10px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
            overflow: hidden;
        }

        header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }

        header h1 {
            font-size: 2.5em;
            margin-bottom: 10px;
            font-weight: 300;
        }

        .subtitle {
            opacity: 0.9;
            font-size: 1.1em;
        }

        .tabs {
            display: flex;
            background: #f8f9fa;
            border-bottom: 1px solid #dee2e6;
        }

        .tab-button {
            flex: 1;
            padding: 15px 20px;
            border: none;
            background: transparent;
            cursor: pointer;
            font-size: 1em;
            transition: all 0.3s ease;
            border-bottom: 3px solid transparent;
        }

        .tab-button:hover {
            background: #e9ecef;
        }

        .tab-button.active {
            background: white;
            border-bottom-color: #667eea;
            color: #667eea;
            font-weight: 600;
        }

        .tab-content {
            display: none;
            padding: 30px;
        }

        .tab-content.active {
            display: block;
        }

        .tab-content h2 {
            color: #495057;
            margin-bottom: 20px;
            font-size: 1.8em;
            font-weight: 300;
        }

        .tree-container {
            max-height: 600px;
            overflow-y: auto;
            border: 1px solid #dee2e6;
            border-radius: 5px;
            background: #f8f9fa;
            padding: 15px;
        }

        .tree-node {
            margin: 2px 0;
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            font-size: 0.9em;
        }

        .tree-node-header {
            display: flex;
            align-items: center;
            padding: 8px 12px;
            border-radius: 4px;
            transition: all 0.2s ease;
            gap: 8px;
        }

        .tree-node-header.clickable {
            cursor: pointer;
        }

        .tree-node-header.clickable:hover {
            background: rgba(102, 126, 234, 0.1);
            transform: translateX(2px);
        }

        .tree-node-header:not(.clickable):hover {
            background: rgba(108, 117, 125, 0.1);
        }

        .tree-expand-icon {
            color: #6c757d;
            font-size: 0.8em;
            transition: transform 0.2s ease;
            width: 12px;
            text-align: center;
        }

        .tree-expand-icon.expanded {
            transform: rotate(90deg);
        }

        .tree-leaf-icon {
            color: #6c757d;
            font-size: 0.6em;
            width: 12px;
            text-align: center;
        }

        .operation-name {
            color: #28a745;
            font-weight: 600;
            flex: 1;
        }

        .timestamp {
            color: #6c757d;
            font-size: 0.85em;
            margin-left: auto;
        }

        .tree-children {
            margin-left: 20px;
            border-left: 2px solid #e9ecef;
            padding-left: 8px;
            margin-top: 4px;
        }

        .tree-children.show {
            display: block !important;
        }

        .tree-node-header.threshold-exceeded-tree {
            background-color: #ffebee !important;
            border-left: 4px solid #f44336 !important;
            border-radius: 4px !important;
            box-shadow: 0 2px 4px rgba(244, 67, 54, 0.2) !important;
        }

        .tree-node-header.threshold-exceeded-tree .operation-name {
            color: #c62828 !important;
            font-weight: 700 !important;
        }

        .tree-node-header.threshold-exceeded-tree:hover {
            background-color: #ffcdd2 !important;
            transform: translateX(4px) !important;
            box-shadow: 0 4px 8px rgba(244, 67, 54, 0.3) !important;
        }

        .duration-badge.threshold-exceeded-badge {
            background: #f44336 !important;
            color: white !important;
            font-weight: bold !important;
            animation: pulse 2s infinite !important;
        }

        @keyframes pulse {
            0% { transform: scale(1); }
            50% { transform: scale(1.05); }
            100% { transform: scale(1); }
        }

        kbd {
            background: #f8f9fa;
            border: 1px solid #dee2e6;
            border-radius: 3px;
            padding: 2px 4px;
            font-size: 0.8em;
            font-family: monospace;
        }

        .duration-badge {
            background: #667eea;
            color: white;
            padding: 2px 8px;
            border-radius: 12px;
            font-size: 0.8em;
            margin-left: 10px;
        }

        .table-container {
            overflow-x: auto;
        }

        table {
            width: 100%;
            border-collapse: collapse;
            background: white;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }

        th, td {
            padding: 12px 15px;
            text-align: left;
            border-bottom: 1px solid #dee2e6;
        }

        th {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            font-weight: 600;
            text-transform: uppercase;
            font-size: 0.9em;
            letter-spacing: 0.5px;
        }

        tr:hover {
            background: #f8f9fa;
        }

        tr.threshold-exceeded {
            background-color: #ffebee !important;
            color: #c62828 !important;
            font-weight: 700 !important;
            border-left: 4px solid #f44336 !important;
            position: relative !important;
        }

        tr.threshold-exceeded:hover {
            background-color: #ffcdd2 !important;
            transform: translateX(2px);
            box-shadow: 0 2px 4px rgba(244, 67, 54, 0.3);
        }

        tr.threshold-exceeded td {
            border-top: 1px solid #f44336 !important;
            border-bottom: 1px solid #f44336 !important;
        }

        tr.threshold-exceeded td:first-child {
            border-left: none !important;
        }

        tr.threshold-exceeded td:last-child {
            border-right: 1px solid #f44336 !important;
        }

        .number {
            text-align: right;
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
        }

        .ranking-container {
            display: grid;
            gap: 15px;
        }

        .rank-item {
            background: white;
            border: 1px solid #dee2e6;
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.05);
            transition: transform 0.2s ease, box-shadow 0.2s ease;
        }

        .rank-item:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0,0,0,0.1);
        }

        .rank-number {
            display: inline-block;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            width: 30px;
            height: 30px;
            border-radius: 50%;
            text-align: center;
            line-height: 30px;
            font-weight: bold;
            margin-right: 15px;
        }

        .operation-name {
            font-weight: 600;
            color: #495057;
            font-size: 1.1em;
        }

        .operation-details {
            color: #6c757d;
            margin-top: 5px;
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            font-size: 0.9em;
        }

        @media (max-width: 768px) {
            .container {
                margin: 10px;
                border-radius: 5px;
            }
            
            header {
                padding: 20px;
            }
            
            header h1 {
                font-size: 2em;
            }
            
            .tab-content {
                padding: 20px;
            }
            
            .tabs {
                flex-direction: column;
            }
        }
        "#.to_string()
    }

    fn generate_threshold_info(&self, threshold_seconds: Option<f64>) -> String {
        if let Some(threshold) = threshold_seconds {
            format!(r#"
            <div style="background: #fff3cd; border: 1px solid #ffeaa7; border-radius: 5px; padding: 12px; margin-bottom: 15px; font-size: 0.9em;">
                <strong>🚨 Threshold Alert:</strong> Operations with total duration > <strong>{:.3}s</strong> are highlighted in <span style="color: #dc3545; font-weight: bold;">red</span>
            </div>
            "#, threshold)
        } else {
            String::new()
        }
    }

    fn generate_html_tree(&self, threshold_seconds: Option<f64>) -> String {
        #[derive(Debug)]
        struct TreeNode {
            operation: String,
            timestamp: f64,
            duration: Option<f64>,
            children: Vec<TreeNode>,
        }

        fn build_tree_recursive(entries: &[CallStackEntry], start_index: &mut usize) -> Vec<TreeNode> {
            let mut nodes = Vec::new();
            
            while *start_index < entries.len() {
                let entry = &entries[*start_index];
                
                if entry.is_enter {
                    let _enter_index = *start_index;
                    *start_index += 1;
                    
                    // Build children for this enter operation
                    let children = build_tree_recursive(entries, start_index);
                    
                    // Find the corresponding exit and calculate duration
                    let duration = if *start_index < entries.len() && 
                                   !entries[*start_index].is_enter && 
                                   entries[*start_index].operation == entry.operation {
                        let exit_time = entries[*start_index].timestamp;
                        *start_index += 1; // Consume the exit
                        Some(exit_time - entry.timestamp)
                    } else {
                        None
                    };
                    
                    nodes.push(TreeNode {
                        operation: entry.operation.clone(),
                        timestamp: entry.timestamp,
                        duration,
                        children,
                    });
                } else {
                    // Exit without matching enter, return to parent
                    break;
                }
            }
            
            nodes
        }

        fn render_tree_node(node: &TreeNode, node_id: &str, threshold_seconds: Option<f64>) -> String {
            let has_children = !node.children.is_empty();
            
            // Check if this node exceeds the threshold
            let is_over_threshold = if let Some(duration) = node.duration {
                threshold_seconds
                    .map(|threshold| duration > threshold)
                    .unwrap_or(false)
            } else {
                false
            };
            
            let duration_text = if let Some(duration) = node.duration {
                let badge_class = if is_over_threshold { "duration-badge threshold-exceeded-badge" } else { "duration-badge" };
                format!("<span class=\"{}\">{:.3}ms</span>", badge_class, duration * 1000.0)
            } else {
                String::new()
            };

            let expand_icon = if has_children {
                "<span class=\"tree-expand-icon\">▶</span>"
            } else {
                "<span class=\"tree-leaf-icon\">•</span>"
            };

            let tree_node_class = if is_over_threshold {
                if has_children { " has-children threshold-exceeded-tree" } else { " threshold-exceeded-tree" }
            } else {
                if has_children { " has-children" } else { "" }
            };

            let mut html = format!(
                "<div class=\"tree-node{}\" data-node-id=\"{}\">\n",
                tree_node_class,
                node_id
            );

            let header_class = if is_over_threshold {
                if has_children { " clickable threshold-exceeded-tree" } else { " threshold-exceeded-tree" }
            } else {
                if has_children { " clickable" } else { "" }
            };

            html.push_str(&format!(
                "  <div class=\"tree-node-header{}\" onclick=\"toggleNode('{}')\">\n",
                header_class,
                node_id
            ));

            let operation_display = if is_over_threshold {
                format!("🚨 {}", node.operation)
            } else {
                node.operation.clone()
            };

            html.push_str(&format!(
                "    {}<span class=\"operation-name\">{}</span>\n    <span class=\"timestamp\">at {:.6}s</span>{}\n",
                expand_icon,
                operation_display,
                node.timestamp,
                duration_text
            ));

            html.push_str("  </div>\n");

            if has_children {
                html.push_str(&format!(
                    "  <div class=\"tree-children\" id=\"children-{}\" style=\"display: none;\">\n",
                    node_id
                ));

                for (i, child) in node.children.iter().enumerate() {
                    let child_id = format!("{}-{}", node_id, i);
                    html.push_str(&render_tree_node(child, &child_id, threshold_seconds));
                }

                html.push_str("  </div>\n");
            }

            html.push_str("</div>\n");
            html
        }

        let mut index = 0;
        let tree = build_tree_recursive(&self.entries, &mut index);
        
        let mut html = String::new();
        for (i, node) in tree.iter().enumerate() {
            html.push_str(&render_tree_node(node, &format!("node-{}", i), threshold_seconds));
        }
        
        html
    }

    fn generate_html_stats_table(&self, threshold_seconds: Option<f64>) -> String {
        let mut html = String::from(r#"
        <table>
            <thead>
                <tr><th>Operation</th><th>Count</th><th>Total (s)</th><th>Average (s)</th><th>Min (s)</th><th>Max (s)</th></tr>
            </thead>
            <tbody>
        "#);

        let mut sorted_stats: Vec<_> = self.operation_stats.iter().collect();
        sorted_stats.sort_by(|a, b| b.1.total_duration.partial_cmp(&a.1.total_duration).unwrap());

        for (operation, stats) in sorted_stats {
            let is_over_threshold = threshold_seconds
                .map(|threshold| stats.total_duration > threshold)
                .unwrap_or(false);
            
            let row_class = if is_over_threshold { " class=\"threshold-exceeded\"" } else { "" };
            let operation_display = if is_over_threshold {
                format!("🚨 {}", operation)
            } else {
                operation.to_string()
            };

            html.push_str(&format!(r#"
                <tr{}>
                    <td>{}</td>
                    <td class="number">{}</td>
                    <td class="number">{:.6}</td>
                    <td class="number">{:.6}</td>
                    <td class="number">{:.6}</td>
                    <td class="number">{:.6}</td>
                </tr>
            "#,
                row_class, operation_display, stats.call_count, stats.total_duration,
                stats.average_duration(),
                if stats.min_duration == f64::MAX { 0.0 } else { stats.min_duration },
                if stats.max_duration == f64::MIN { 0.0 } else { stats.max_duration }
            ));
        }

        html.push_str("</tbody></table>");
        html
    }

    fn generate_html_top_total(&self) -> String {
        let mut html = String::new();
        let longest_total = self.find_longest_operations_by_total(10);
        
        for (i, (operation, stats)) in longest_total.iter().enumerate() {
            html.push_str(&format!(r#"
                <div class="rank-item">
                    <span class="rank-number">{}</span>
                    <div style="display: inline-block;">
                        <div class="operation-name">{}</div>
                        <div style="color: #6c757d; margin-top: 5px; font-family: Monaco, monospace; font-size: 0.9em;">
                            {:.6}s total • {} calls • avg: {:.6}s
                        </div>
                    </div>
                </div>
            "#, i + 1, operation, stats.total_duration, stats.call_count, stats.average_duration()));
        }
        
        html
    }

    fn generate_html_top_average(&self) -> String {
        let mut html = String::new();
        let longest_avg = self.find_longest_operations_by_average(20);
        let mut count = 0;
        
        for (operation, stats) in longest_avg.iter() {
            if stats.call_count >= 5 && count < 10 {
                count += 1;
                html.push_str(&format!(r#"
                    <div class="rank-item">
                        <span class="rank-number">{}</span>
                        <div style="display: inline-block;">
                            <div class="operation-name">{}</div>
                            <div class="operation-details">
                                {:.6}s avg • {} calls • total: {:.6}s
                            </div>
                        </div>
                    </div>
                "#,
                    count,
                    operation,
                    stats.average_duration(),
                    stats.call_count,
                    stats.total_duration
                ));
            }
        }
        
        html
    }

    fn find_longest_operations_by_average(&self, top_n: usize) -> Vec<(&String, &OperationStats)> {
        let mut sorted: Vec<_> = self.operation_stats.iter().collect();
        sorted.sort_by(|a, b| b.1.average_duration().partial_cmp(&a.1.average_duration()).unwrap());
        sorted.into_iter().take(top_n).collect()
    }

    fn generate_javascript(&self) -> String {
        r#"
        function showTab(tabName) {
            // Hide all tab contents
            const contents = document.querySelectorAll('.tab-content');
            contents.forEach(content => content.classList.remove('active'));
            
            // Remove active class from all buttons
            const buttons = document.querySelectorAll('.tab-button');
            buttons.forEach(button => button.classList.remove('active'));
            
            // Show selected tab content
            document.getElementById(tabName).classList.add('active');
            
            // Add active class to clicked button
            event.target.classList.add('active');
        }

        function toggleNode(nodeId) {
            const childrenContainer = document.getElementById('children-' + nodeId);
            const expandIcon = document.querySelector(`[data-node-id="${nodeId}"] .tree-expand-icon`);
            
            if (childrenContainer) {
                const isVisible = childrenContainer.style.display !== 'none';
                
                if (isVisible) {
                    // Collapse
                    childrenContainer.style.display = 'none';
                    expandIcon.classList.remove('expanded');
                } else {
                    // Expand
                    childrenContainer.style.display = 'block';
                    expandIcon.classList.add('expanded');
                }
            }
        }

        // Initialize tree functionality
        document.addEventListener('DOMContentLoaded', function() {
            // Add expand/collapse all functionality
            const treeContainer = document.querySelector('.tree-container');
            
            // Add control buttons
            const controlsDiv = document.createElement('div');
            controlsDiv.style.cssText = 'margin-bottom: 10px; display: flex; gap: 10px;';
            controlsDiv.innerHTML = `
                <button onclick="expandAll()" style="padding: 5px 10px; border: 1px solid #dee2e6; background: white; border-radius: 3px; cursor: pointer; font-size: 0.8em;">
                    Expand All
                </button>
                <button onclick="collapseAll()" style="padding: 5px 10px; border: 1px solid #dee2e6; background: white; border-radius: 3px; cursor: pointer; font-size: 0.8em;">
                    Collapse All
                </button>
            `;
            
            treeContainer.parentNode.insertBefore(controlsDiv, treeContainer);
        });

        function expandAll() {
            const allChildren = document.querySelectorAll('.tree-children');
            const allIcons = document.querySelectorAll('.tree-expand-icon');
            
            allChildren.forEach(child => {
                child.style.display = 'block';
            });
            
            allIcons.forEach(icon => {
                icon.classList.add('expanded');
            });
        }

        function collapseAll() {
            const allChildren = document.querySelectorAll('.tree-children');
            const allIcons = document.querySelectorAll('.tree-expand-icon');
            
            allChildren.forEach(child => {
                child.style.display = 'none';
            });
            
            allIcons.forEach(icon => {
                icon.classList.remove('expanded');
            });
        }

        // Add keyboard shortcuts
        document.addEventListener('keydown', function(e) {
            if (e.ctrlKey || e.metaKey) {
                switch(e.key) {
                    case 'e':
                        e.preventDefault();
                        expandAll();
                        break;
                    case 'c':
                        e.preventDefault();
                        collapseAll();
                        break;
                }
            }
        });
        "#.to_string()
    }
}

/// Reads callstack data from a ZIP file
/// 
/// **Rust Concepts Demonstrated:**
/// - **ZIP Archive Processing** - Reading compressed files in memory
/// - **Error Handling** - Comprehensive error propagation with context
/// - **Pattern Matching** - Using file extensions to identify callstack files
fn read_callstack_from_zip(zip_data: &[u8]) -> Result<String, String> {
    let cursor = Cursor::new(zip_data);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to open ZIP archive: {}", e))?;

    // Try to find a file that might contain callstack data
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to access file at index {}: {}", i, e))?;
        
        let name = file.name().to_string();
        
        // Check if this looks like a callstack file
        if name.ends_with(".txt") || name.ends_with(".log") || name.ends_with(".trace") 
           || name.contains("callstack") || name.contains("trace") || name.contains("log") {
            
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| format!("Failed to read file '{}': {}", name, e))?;
            
            // Check if the content looks like callstack data
            if contents.contains("Enter") && contents.contains("Exit") {
                info!("Found callstack data in file: {}", name);
                return Ok(contents);
            }
        }
    }

    // If no specific file found, try the first text-like file
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to access file at index {}: {}", i, e))?;
        
        let name = file.name().to_string();
        
        if !name.ends_with("/") {  // Not a directory
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if !contents.trim().is_empty() {
                    info!("Using file: {}", name);
                    return Ok(contents);
                }
            }
        }
    }

    Err("No suitable callstack data found in zip file".to_string())
}

/// Creates a ZIP file containing the HTML visualization report
/// 
/// **Rust Concepts Demonstrated:**
/// - **In-Memory ZIP Creation** - Building compressed files without file system
/// - **Error Handling** - Detailed error context for debugging
/// - **Resource Management** - Proper cleanup of ZIP writer resources
fn create_callstack_html_zip(html_content: &str) -> Result<Vec<u8>, String> {
    let mut zip_buffer = Vec::new();
    
    {
        let cursor = Cursor::new(&mut zip_buffer);
        let mut zip = ZipWriter::new(cursor);
        
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);
        
        let html_filename = "callstack_analysis_report.html";
        
        zip.start_file(html_filename, options)
            .map_err(|e| format!("Failed to start ZIP file entry: {}", e))?;
        
        zip.write_all(html_content.as_bytes())
            .map_err(|e| format!("Failed to write HTML content to ZIP: {}", e))?;
        
        zip.finish()
            .map_err(|e| format!("Failed to finalize ZIP file: {}", e))?;
    }
    
    Ok(zip_buffer)
}

// === DATA STRUCTURES SECTION ===
// Demonstrates Rust structs, derive macros, and type definitions

/// Session represents an active user session in our system
/// 
/// **Rust Concepts Demonstrated:**
/// - `#[derive(...)]` - Automatically implements common traits
/// - `Debug` - Enables printing with {:?} formatter
/// - `Clone` - Allows creating copies of the struct
/// - `Serialize/Deserialize` - Enables JSON conversion via Serde
/// - `struct` - Defines a custom data type with named fields
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Session {
    /// Unique identifier for the session (owned String)
    session_id: String,
    /// When the session was created (using chrono's DateTime)
    created_at: DateTime<Utc>,
    /// Whether the session is currently active
    is_active: bool,
}

/// Generic API response wrapper for consistent JSON responses
/// 
/// **Rust Concepts Demonstrated:**
/// - **Generics** - `<T>` allows this struct to work with any data type
/// - **Option<T>** - Rust's null safety mechanism (no null pointers!)
/// - Only `Serialize` derive (not Deserialize) - this is output-only
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    /// Whether the operation was successful
    success: bool,
    /// Human-readable message describing the result
    message: String,
    /// Optional data payload - uses Option<T> for null safety
    data: Option<T>,
}

/// Status information for a specific session
/// 
/// **Rust Concepts Demonstrated:**
/// - Struct with `Option` fields for nullable data
/// - Different field types showing Rust's type system
#[derive(Debug, Serialize)]
struct SessionStatus {
    session_id: String,
    is_valid: bool,
    /// Optional timestamp - Some(datetime) if valid, None if invalid
    created_at: Option<DateTime<Utc>>,
}

/// Main application configuration loaded from TOML file
/// 
/// **Rust Concepts Demonstrated:**
/// - **Nested structs** - Config contains other struct types
/// - Only `Deserialize` derive - this is input-only from config file
/// - Composition pattern in Rust
#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
    logging: LoggingConfig,
}

/// Server-specific configuration settings
#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    /// u16 represents an unsigned 16-bit integer (0-65535) - perfect for port numbers
    port: u16,
}

/// Logging configuration settings
#[derive(Debug, Deserialize)]
struct LoggingConfig {
    log_file: String,
    log_level: String,
    /// bool type for true/false values
    console_logging: bool,
}

// === TYPE ALIASES SECTION ===
// Demonstrates type aliases for complex generic types

/// Type alias for our session storage mechanism
/// 
/// **Rust Concepts Demonstrated:**
/// - **Type Alias** - Creates a shorter name for a complex type
/// - **Arc<T>** - Atomic Reference Counting for shared ownership across threads
/// - **Mutex<T>** - Mutual exclusion for thread-safe interior mutability
/// - **HashMap<K,V>** - Hash map data structure for key-value storage
/// - **Thread Safety** - Arc + Mutex = safely share mutable data between threads
type SessionStore = Arc<Mutex<HashMap<String, Session>>>;

// === MIDDLEWARE SECTION ===
// Demonstrates custom middleware implementation using Actix-web traits

/// Custom middleware for logging HTTP requests and responses
/// 
/// **Rust Concepts Demonstrated:**
/// - **Struct** with public visibility (`pub`)
/// - **Encapsulation** - log_file is private but struct is public
/// - This will implement the `Transform` trait (see below)
pub struct RequestResponseLogger {
    log_file: String,
}

/// Implementation block for RequestResponseLogger
/// 
/// **Rust Concepts Demonstrated:**
/// - **impl blocks** - Where we define methods for our types
/// - **Associated functions** vs **methods** (new is associated, no &self)
/// - **Self** - refers to the type being implemented (RequestResponseLogger)
/// - **Constructor pattern** - `new` is Rust's conventional constructor name
impl RequestResponseLogger {
    /// Constructor function for creating a new logger instance
    /// 
    /// **Parameters:**
    /// - `log_file: String` - Takes ownership of the string
    /// 
    /// **Returns:**
    /// - `Self` - Returns an instance of RequestResponseLogger
    pub fn new(log_file: String) -> Self {
        Self { log_file }  // Struct initialization syntax
    }
}

/// Implementation of the Transform trait for our custom middleware
/// 
/// **Rust Concepts Demonstrated:**
/// - **Trait Implementation** - `impl Trait for Type` syntax
/// - **Generic Parameters** - `<S, B>` make this work with any service/body type
/// - **Where Clauses** - Constraints on generic parameters for type safety
/// - **Associated Types** - Types defined within traits (Response, Error, etc.)
/// - **Lifetimes** - `'static` means the type lives for the entire program duration
/// - **Complex Type Bounds** - Service trait with specific associated types
impl<S, B> Transform<S, ServiceRequest> for RequestResponseLogger
where
    // These where clauses constrain our generic parameters:
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,  // The service's future must live for 'static lifetime
    B: 'static,          // The body type must also be 'static
{
    // Associated types - defining what types this trait implementation uses
    type Response = ServiceResponse<B>;                      // Response type passes through
    type Error = Error;                                      // Error type passes through  
    type InitError = ();                                     // No initialization errors
    type Transform = RequestResponseLoggerMiddleware<S>;     // The actual middleware type
    type Future = Ready<Result<Self::Transform, Self::InitError>>; // Future that resolves immediately

    /// Creates a new middleware instance wrapping the given service
    /// 
    /// **Rust Concepts:**
    /// - **Method** - takes `&self` (borrowed reference)
    /// - **Ownership Transfer** - service parameter moves into middleware
    /// - **ready()** - Creates a future that resolves immediately
    /// - **Result<T, E>** - Error handling type
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestResponseLoggerMiddleware {
            service,                           // Moves service into the middleware
            log_file: self.log_file.clone(),  // Clones the log file path
        }))
    }
}

/// The actual middleware that wraps around each HTTP service
/// 
/// **Rust Concepts Demonstrated:**
/// - **Generic Struct** - `<S>` allows this to wrap any service type
/// - **Composition Pattern** - Contains the wrapped service
pub struct RequestResponseLoggerMiddleware<S> {
    service: S,       // The wrapped service (generic type S)
    log_file: String, // Configuration for this middleware instance
}

/// Implementation of Service trait for our middleware
/// 
/// **Rust Concepts Demonstrated:**
/// - **Complex Trait Implementation** with multiple generic bounds
/// - **Pin<Box<dyn Future>>** - Type erasure for different future types
/// - **Macro Usage** - forward_ready! delegates readiness checks
/// - **Async Programming** - Handling HTTP requests asynchronously
impl<S, B> Service<ServiceRequest> for RequestResponseLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    /// **Complex Future Type** - Pin<Box<dyn Future>> enables type erasure
    /// This allows different async block types to be returned from the same function
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    // **Macro Usage** - Delegates the ready() check to the wrapped service
    // This is Actix-web specific - forwards readiness state
    forward_ready!(service);

    /// Core request handling method - called for every HTTP request
    /// 
    /// **Rust Concepts:**
    /// - **Method** - takes `&self` (immutable borrow)
    /// - **Ownership** - `req` parameter is moved (owned)
    /// - **Returns Future** - enabling async processing
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // **Cloning for Move Semantics** - Clone data we need in the async block
        // Since the async block takes ownership, we must clone before moving
        let log_file = self.log_file.clone();
        let method = req.method().clone();        // Clone HTTP method
        let uri = req.uri().clone();              // Clone URI 
        let timestamp = Utc::now();               // Capture current time
        let peer_addr = req.peer_addr();          // Get client address (Option<SocketAddr>)
        
        // **String Formatting** - format! macro creates owned String
        let request_log = format!(
            "[{}] REQUEST: {} {} from {:?}",
            timestamp.format("%Y-%m-%d %H:%M:%S UTC"),  // DateTime formatting
            method,
            uri,
            peer_addr  // Option<SocketAddr> - prints as Some(addr) or None
        );
        
        // **Logging** - Using log macros for structured logging
        info!("{}", request_log);
        log_to_file(&log_file, &request_log);

        // **Service Call** - Call the wrapped service, gets a Future
        let fut = self.service.call(req);

        // **Box::pin + async move** - Complex async pattern explanation:
        // 1. `async move` - Takes ownership of captured variables
        // 2. `Box::pin` - Heap allocates and pins the future (required for dyn Future)
        // 3. This pattern allows type erasure while maintaining async behavior
        Box::pin(async move {
            // **Await Operation** - Wait for the wrapped service to complete
            // **? Operator** - Early return if Result is Err, otherwise unwrap Ok value
            let res = fut.await?;
            let status = res.status();
            
            // **Response Logging** - Log after processing
            let response_log = format!(
                "[{}] RESPONSE: {} {} -> {} {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                method,    // We moved this into the async block
                uri,       // We moved this into the async block
                status.as_u16(),  // Convert StatusCode to u16
                status.canonical_reason().unwrap_or("Unknown")  // Option handling
            );
            
            info!("{}", response_log);
            log_to_file(&log_file, &response_log);

            // **Return Result** - Ok wraps the successful response
            Ok(res)
        })
    }
}

// === UTILITY FUNCTIONS SECTION ===

/// Writes a log message to the specified file, creating directories as needed
/// 
/// **Rust Concepts Demonstrated:**
/// - **Function Parameters** - `&str` means borrowed string slice (no ownership transfer)
/// - **Option Handling** - `if let Some(x) = option` pattern
/// - **Result Handling** - Multiple error handling patterns
/// - **Method Chaining** - Builder pattern with OpenOptions
/// - **Early Return** - Using `return` for control flow
fn log_to_file(log_file: &str, message: &str) {
    // **Option Pattern Matching** - parent() returns Option<&Path>
    // if let Some(x) = option unwraps if Some, skips if None
    if let Some(parent) = Path::new(log_file).parent() {
        // **Result Handling with if let** - Handle potential directory creation error
        if let Err(e) = std::fs::create_dir_all(parent) {
            error!("Failed to create log directory: {}", e);
            return;  // **Early Return** - Exit function on error
        }
    }
    
    // **Match Expression** - Comprehensive Result handling
    // OpenOptions uses the **Builder Pattern** - method chaining for configuration
    match OpenOptions::new()
        .create(true)    // Create file if it doesn't exist
        .append(true)    // Append to existing content
        .open(log_file)  // Returns Result<File, Error>
    {
        // **Destructuring Success** - mut file gives mutable access
        Ok(mut file) => {
            // **Nested Error Handling** - writeln! can also fail
            if let Err(e) = writeln!(file, "{}", message) {
                error!("Failed to write to log file: {}", e);
            }
        }
        // **Destructuring Failure** - Extract error for logging
        Err(e) => {
            error!("Failed to open log file {}: {}", log_file, e);
        }
    }
}

// === IMPLEMENTATION BLOCKS FOR HELPER METHODS ===

/// Implementation of helper methods for ApiResponse
/// 
/// **Rust Concepts Demonstrated:**
/// - **Generic impl blocks** - `impl<T>` works for any type T
/// - **Associated functions** - Functions that don't take &self
/// - **Constructor pattern** - Creating instances with specific configurations
/// - **String ownership** - Converting &str to owned String
impl<T> ApiResponse<T> {
    /// Creates a successful API response
    /// 
    /// **Parameters:**
    /// - `message: &str` - Borrowed string slice (no ownership needed)
    /// - `data: Option<T>` - Optional data of any type T
    /// 
    /// **Returns:** `Self` - An instance of ApiResponse<T>
    /// 
    /// **Rust Concepts:**
    /// - **Generic function** - Works with any type T
    /// - **Option<T>** - Rust's way of handling nullable values safely
    fn success(message: &str, data: Option<T>) -> Self {
        Self {
            success: true,
            message: message.to_string(),  // **to_string()** - Convert &str to owned String
            data,                          // **Shorthand** - same as data: data
        }
    }

    /// Creates an error API response
    /// 
    /// **Note:** No data parameter since errors don't carry successful data
    /// 
    /// **Rust Concepts:**
    /// - **None** - Explicit null value, type-safe
    /// - **Type inference** - Rust infers T from context
    fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,  // **None** - Option's "null" value, but type-safe
        }
    }
}

// === BRM HANDLER FUNCTIONS SECTION ===
// Functions for Oracle BRM field management

/// Load Oracle BRM fields for a specific session
/// 
/// **Rust Concepts Demonstrated:**
/// - **String Body Handling** - Direct string payload processing
/// - **Path Parameters** - Session ID extraction from URL
/// - **Module Function Calls** - Calling BRM module functions
/// - **JSON Response** - Structured API response
async fn load_obrm_fields_handler(
    path: web::Path<String>,
    body: String,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    info!("Loading BRM fields for session: {}", session_id);
    
    // Input validation
    if session_id.trim().is_empty() {
        let response = brm::LoadFieldsResponse {
            success: false,
            message: "Session ID cannot be empty".to_string(),
            field_count: 0,
            session_id: session_id.clone(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    if body.trim().is_empty() {
        let response = brm::LoadFieldsResponse {
            success: false,
            message: "Request body cannot be empty".to_string(),
            field_count: 0,
            session_id: session_id.clone(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Load fields using BRM module
    let result = brm::load_obrm_fields(&session_id, body);
    let fields = brm::get_session_fields(&session_id);
    
    let response = brm::LoadFieldsResponse {
        success: true,
        message: result,
        field_count: fields.len(),
        session_id: session_id.clone(),
    };
    
    info!("BRM fields loaded for session {}: {} fields", session_id, fields.len());
    
    Ok(HttpResponse::Ok().json(response))
}

/// Get all loaded BRM fields for a specific session
/// 
/// **Rust Concepts Demonstrated:**
/// - **Read-only Operations** - Retrieving stored data
/// - **Vec Serialization** - Converting Vec to JSON
/// - **Option Handling** - Dealing with potentially empty data
async fn get_session_fields_handler(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    info!("Getting BRM fields for session: {}", session_id);
    
    let fields = brm::get_session_fields(&session_id);
    
    if fields.is_empty() {
        let response = ApiResponse::<Vec<brm::Field>>::error("No fields loaded for this session");
        return Ok(HttpResponse::NotFound().json(response));
    }
    
    let response = ApiResponse::success("Fields retrieved successfully", Some(fields));
    
    info!("Retrieved {} BRM fields for session {}", response.data.as_ref().map_or(0, |f| f.len()), session_id);
    
    Ok(HttpResponse::Ok().json(response))
}

/// Convert BRM field specification to PODL format (session-based)
/// 
/// **Rust Concepts Demonstrated:**
/// - **String Processing** - Parsing pipe-delimited field specifications
/// - **Text Transformation** - Converting custom format to PODL
/// - **Format Validation** - Ensuring proper input format
/// - **Template Generation** - Creating structured output format
/// - **Session Management** - Validates session exists and has loaded fields
async fn convert_fld_spec_to_podl_handler(
    path: web::Path<String>,
    body: String,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    info!("Converting field specification to PODL format for session: {}", session_id);
    
    // Input validation
    if session_id.trim().is_empty() {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Session ID cannot be empty".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    if body.trim().is_empty() {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Request body cannot be empty".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Check if fields are loaded for this session
    let field_count = brm::G_SESSION_MAP.lock().unwrap()
        .get(&session_id)
        .map_or(0, |m| m.len());
    
    if field_count == 0 {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Please load BRM fields first using the load_obrm_fields endpoint".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Validate input format - each line should have at least 4 pipe-separated parts
    let lines: Vec<&str> = body.lines().filter(|line| !line.trim().is_empty()).collect();
    let mut invalid_lines = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 4 {
            invalid_lines.push(format!("Line {}: {}", i + 1, line));
        }
    }
    
    if !invalid_lines.is_empty() {
        let response = brm::PodlConversionResponse {
            success: false,
            message: format!("Invalid format. Lines should have format 'field_name|description|data_type|field_id'. Invalid lines: {}", invalid_lines.join(", ")),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Convert to PODL format using session-based function
    let podl_content = brm::convert_fld_spec_to_podl(&session_id, body.clone());
    
    // Check if conversion returned an error message
    if podl_content.starts_with("Please load BRM fields first") {
        let response = brm::PodlConversionResponse {
            success: false,
            message: podl_content,
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    let response = brm::PodlConversionResponse {
        success: true,
        message: format!("Successfully converted {} field specifications to PODL format for session {}", lines.len(), session_id),
        podl_content,
    };
    
    info!("Converted {} field specifications to PODL format for session {}", lines.len(), session_id);
    
    Ok(HttpResponse::Ok().json(response))
}

/// Convert BRM class specification to PODL format
/// 
/// **Rust Concepts Demonstrated:**
/// - **Complex String Processing** - Parsing hierarchical class structures
/// - **State Management** - Tracking nested field levels during parsing
/// - **Session Dependencies** - Requiring pre-loaded field definitions
/// - **Structured Output Generation** - Creating complete PODL class definitions
async fn convert_class_spec_to_podl_handler(
    path: web::Path<String>,
    body: String,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    info!("Converting class specification to PODL format for session: {}", session_id);
    
    // Input validation
    if session_id.trim().is_empty() {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Session ID cannot be empty".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    if body.trim().is_empty() {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Request body cannot be empty".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Check if fields are loaded for this session
    let field_count = brm::G_SESSION_MAP.lock().unwrap()
        .get(&session_id)
        .map_or(0, |m| m.len());
    
    if field_count == 0 {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Please load BRM fields first using the load_obrm_fields endpoint".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Convert class specification to PODL format
    let podl_content = brm::convert_class_spec_to_podl(&session_id, body);
    
    if podl_content.starts_with("Please load fields first") || podl_content == "No data found" {
        let response = brm::PodlConversionResponse {
            success: false,
            message: podl_content,
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    let response = brm::PodlConversionResponse {
        success: true,
        message: format!("Successfully converted class specification to PODL format for session {}", session_id),
        podl_content,
    };
    
    info!("Converted class specification to PODL format for session {}", session_id);
    
    Ok(HttpResponse::Ok().json(response))
}

/// Convert FLIST to C code format (session-based)
/// 
/// **Rust Concepts Demonstrated:**
/// - **FLIST Processing** - Parsing Oracle BRM FLIST format
/// - **C Code Generation** - Converting structured data to Oracle BRM C code
/// - **Session Management** - Validates session exists and has loaded fields
async fn convert_flist2code_handler(
    path: web::Path<String>,
    body: String,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    info!("Converting FLIST to C code for session: {}", session_id);
    
    // Input validation
    if session_id.trim().is_empty() {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Session ID cannot be empty".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    if body.trim().is_empty() {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Request body cannot be empty".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Check if fields are loaded for this session
    let field_count = brm::G_SESSION_MAP.lock().unwrap()
        .get(&session_id)
        .map_or(0, |m| m.len());
    
    if field_count == 0 {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Please load BRM fields first using the load_obrm_fields endpoint".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Convert FLIST to C code
    let c_code = brm::convert_flist2code(body.clone());
    
    let response = brm::PodlConversionResponse {
        success: true,
        message: format!("Successfully converted FLIST to C code for session {}", session_id),
        podl_content: c_code,
    };
    
    info!("Converted FLIST to C code for session {}", session_id);
    
    Ok(HttpResponse::Ok().json(response))
}

/// Convert FLIST to XML format (session-based)
/// 
/// **Rust Concepts Demonstrated:**
/// - **FLIST Processing** - Parsing Oracle BRM FLIST format
/// - **XML Generation** - Converting structured data to XML format
/// - **Session Management** - Validates session exists and has loaded fields
async fn convert_flist2xml_handler(
    path: web::Path<String>,
    body: String,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    info!("Converting FLIST to XML for session: {}", session_id);
    
    // Input validation
    if session_id.trim().is_empty() {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Session ID cannot be empty".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    if body.trim().is_empty() {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Request body cannot be empty".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Check if fields are loaded for this session
    let field_count = brm::G_SESSION_MAP.lock().unwrap()
        .get(&session_id)
        .map_or(0, |m| m.len());
    
    if field_count == 0 {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Please load BRM fields first using the load_obrm_fields endpoint".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Convert FLIST to XML
    let xml_content = brm::convert_flist2xml(body.clone());
    
    let response = brm::PodlConversionResponse {
        success: true,
        message: format!("Successfully converted FLIST to XML for session {}", session_id),
        podl_content: xml_content,
    };
    
    info!("Converted FLIST to XML for session {}", session_id);
    
    Ok(HttpResponse::Ok().json(response))
}

/// Convert FLIST to JSON format (session-based)
/// 
/// **Rust Concepts Demonstrated:**
/// - **FLIST Processing** - Parsing Oracle BRM FLIST format
/// - **JSON Generation** - Converting structured data to JSON format via XML
/// - **Session Management** - Validates session exists and has loaded fields
async fn convert_flist2json_handler(
    path: web::Path<String>,
    body: String,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    info!("Converting FLIST to JSON for session: {}", session_id);
    
    // Input validation
    if session_id.trim().is_empty() {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Session ID cannot be empty".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    if body.trim().is_empty() {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Request body cannot be empty".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Check if fields are loaded for this session
    let field_count = brm::G_SESSION_MAP.lock().unwrap()
        .get(&session_id)
        .map_or(0, |m| m.len());
    
    if field_count == 0 {
        let response = brm::PodlConversionResponse {
            success: false,
            message: "Please load BRM fields first using the load_obrm_fields endpoint".to_string(),
            podl_content: String::new(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // Convert FLIST to JSON
    let json_content = brm::convert_flist2json(body.clone());
    
    let response = brm::PodlConversionResponse {
        success: true,
        message: format!("Successfully converted FLIST to JSON for session {}", session_id),
        podl_content: json_content,
    };
    
    info!("Converted FLIST to JSON for session {}", session_id);
    
    Ok(HttpResponse::Ok().json(response))
}

/// View Call Stack from ZIP file (session-based) - Returns ZIP with HTML visualization
/// 
/// **Rust Concepts Demonstrated:**
/// - **Multipart Form Processing** - Handling file uploads with optional parameters
/// - **ZIP Archive Processing** - Extracting callstack data from compressed files
/// - **Regex Pattern Matching** - Parsing Oracle BRM callstack format
/// - **Performance Analysis** - Calculating operation durations and statistics
/// - **Interactive HTML Generation** - Creating dynamic web visualizations with threshold highlighting
/// - **ZIP File Response** - Returning compressed HTML reports as downloads
async fn view_call_stack_handler(
    mut payload: Multipart,
) -> Result<HttpResponse> {
    info!("Processing call stack visualization request");
    
    let mut zip_data: Vec<u8> = Vec::new();
    let mut highlight_threshold: Option<f64> = None;

    // Process multipart form data
    while let Some(mut field) = payload.try_next().await? {
        let field_name = field.name();
        
        match field_name {
            "file" => {
                // Process ZIP file upload
                info!("Processing uploaded ZIP file");
                while let Some(chunk) = field.try_next().await? {
                    zip_data.extend_from_slice(&chunk);
                }
                info!("Received ZIP file with {} bytes", zip_data.len());
            }
            "threshold" | "highlight_threshold" => {
                // Process threshold parameter
                let mut threshold_data = Vec::new();
                while let Some(chunk) = field.try_next().await? {
                    threshold_data.extend_from_slice(&chunk);
                }
                if let Ok(threshold_str) = String::from_utf8(threshold_data) {
                    match threshold_str.trim().parse::<f64>() {
                        Ok(threshold) if threshold > 0.0 => {
                            highlight_threshold = Some(threshold);
                            info!("Threshold set to: {:.6}s", threshold);
                        }
                        Ok(_) => {
                            info!("Invalid threshold value (must be > 0): {}", threshold_str);
                        }
                        Err(_) => {
                            info!("Failed to parse threshold: {}", threshold_str);
                        }
                    }
                }
            }
            _ => {
                // Skip unknown fields
                while let Some(_chunk) = field.try_next().await? {
                    // Consume field data
                }
            }
        }
    }

    // Validate that we received a ZIP file
    if zip_data.is_empty() {
        error!("No ZIP file uploaded or file is empty");
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "No ZIP file uploaded or file is empty",
            "details": "Please upload a valid ZIP file containing callstack data"
        })));
    }

    // Extract callstack data from ZIP
    let callstack_content = match read_callstack_from_zip(&zip_data) {
        Ok(content) => {
            info!("Successfully extracted callstack data ({} bytes)", content.len());
            content
        }
        Err(error) => {
            error!("Failed to extract callstack from ZIP: {}", error);
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to extract callstack from ZIP: {}", error),
                "details": "Ensure the ZIP contains a .txt, .log, or .trace file with Enter/Exit statements"
            })));
        }
    };

    // Create and configure the analyzer
    let mut analyzer = CallStackAnalyzer::new();

    // Parse the callstack data
    if let Err(parse_error) = analyzer.parse_callstack(&callstack_content) {
        error!("Failed to parse callstack: {}", parse_error);
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": format!("Failed to parse callstack: {}", parse_error),
            "details": "Ensure the callstack follows the format: timestamp dots Enter/Exit operation (returncode)"
        })));
    }

    // Calculate operation durations for analysis
    analyzer.calculate_durations();
    
    info!("Parsed {} callstack entries", analyzer.entries.len());
    info!("Analyzed {} unique operations", analyzer.operation_stats.len());

    // Generate HTML report with optional threshold highlighting
    let html_content = analyzer.generate_html_report(highlight_threshold);

    // Create ZIP file containing the HTML report
    match create_callstack_html_zip(&html_content) {
        Ok(zip_bytes) => {
            let filename = format!("callstack_analysis_report_{}.zip", 
                chrono::Utc::now().format("%Y%m%d_%H%M%S"));
            
            info!("Generated callstack analysis ZIP ({} bytes): {}", zip_bytes.len(), filename);
            
            Ok(HttpResponse::Ok()
                .content_type("application/zip")
                .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .append_header(("Content-Length", zip_bytes.len()))
                .body(zip_bytes))
        }
        Err(zip_error) => {
            error!("Failed to create ZIP file: {}", zip_error);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to create ZIP file: {}", zip_error),
                "details": "Internal error during ZIP file creation"
            })))
        }
    }
}


// === HTTP HANDLER FUNCTIONS SECTION ===
// Demonstrates async functions, dependency injection, and error handling

/// Registers a new session in the system
/// 
/// **Rust Concepts Demonstrated:**
/// - **Async functions** - `async fn` enables non-blocking I/O
/// - **Dependency Injection** - Actix-web extracts dependencies from app state
/// - **Path Parameters** - web::Path extracts URL parameters
/// - **Shared State** - web::Data provides thread-safe access to app state
/// - **Result<T, E>** - Return type for operations that can fail
/// - **Lock/Unlock Pattern** - Thread-safe access to shared mutable data
async fn register_session(
    path: web::Path<String>,           // **Path Extraction** - Gets {session_id} from URL
    sessions: web::Data<SessionStore>, // **Shared State** - Thread-safe access to HashMap
) -> Result<HttpResponse> {            // **Result Type** - Can return HTTP response or error
    
    // **into_inner()** - Extracts the String from web::Path wrapper
    let session_id = path.into_inner();
    
    info!("Registering session: {}", session_id);
    
    // **Input Validation** - Check for empty session ID
    if session_id.trim().is_empty() {
        // **Turbofish Syntax** - ApiResponse::<()> explicitly specifies type parameter
        // () is the unit type - represents "no data"
        let response = ApiResponse::<()>::error("Session ID cannot be empty");
        info!("Registration failed - empty session ID. Response: {:?}", response);
        // **Early Return** - Return HTTP 400 Bad Request
        return Ok(HttpResponse::BadRequest().json(response));
    }

    // **Lock Acquisition** - Get exclusive access to the shared HashMap
    // **unwrap()** - Panic if lock is poisoned (usually safe in this context)
    let mut sessions_map = sessions.lock().unwrap();
    
    // **Duplicate Check** - Ensure session doesn't already exist
    if sessions_map.contains_key(&session_id) {
        let response = ApiResponse::<()>::error("Session already exists");
        info!("Registration failed - session {} already exists. Response: {:?}", session_id, response);
        // **HTTP Status Codes** - 409 Conflict for duplicate resources
        return Ok(HttpResponse::Conflict().json(response));
    }

    // **Struct Construction** - Create new Session instance
    let session = Session {
        session_id: session_id.clone(),  // **Clone** - Create owned copy for storage
        created_at: Utc::now(),          // **Current Timestamp**
        is_active: true,                 // **Boolean Literal**
    };

    // **HashMap Insertion** - Store the session
    // Both key and value are cloned to satisfy ownership requirements
    sessions_map.insert(session_id.clone(), session.clone());
    
    // **Success Response** - Some(session) wraps the session data
    let response = ApiResponse::success("Session registered successfully", Some(session));
    info!("Session {} registered successfully. Response: {:?}", session_id, response);
    
    // **HTTP 200 OK** - Successful registration
    Ok(HttpResponse::Ok().json(response))
}

/// Checks the status of an existing session
/// 
/// **Rust Concepts Demonstrated:**
/// - **Advanced Pattern Matching** - Multiple match arms with guard conditions
/// - **Option Handling** - Working with HashMap::get() return values
/// - **Guard Conditions** - `if session.is_active` in match arms
/// - **Wildcard Patterns** - Using `_` to ignore values
/// - **Immutable Borrow** - Reading without modifying shared state
async fn get_session_status(
    path: web::Path<String>,
    sessions: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    info!("Checking status for session: {}", session_id);
    
    // **Immutable Lock** - Only reading, so no `mut` needed
    let sessions_map = sessions.lock().unwrap();
    
    // **Complex Pattern Matching** - Demonstrates Rust's powerful match expressions
    match sessions_map.get(&session_id) {
        // **Guard Condition** - `if session.is_active` adds extra condition to pattern
        // This matches: Some(session) AND session.is_active == true
        Some(session) if session.is_active => {
            let status = SessionStatus {
                session_id: session_id.clone(),
                is_valid: true,
                // **Some() Wrapping** - Converts value to Option
                created_at: Some(session.created_at),
            };
            let response = ApiResponse::success("Session is valid", Some(status));
            info!("Session {} is valid. Response: {:?}", session_id, response);
            Ok(HttpResponse::Ok().json(response))
        }
        // **Wildcard Pattern** - Some(_) matches any Some value
        // This catches inactive sessions (is_active == false)
        Some(_) => {
            let status = SessionStatus {
                session_id: session_id.clone(),
                is_valid: false,
                created_at: None,  // **None** - No timestamp for inactive sessions
            };
            let response = ApiResponse::success("Session exists but is inactive", Some(status));
            info!("Session {} exists but is inactive. Response: {:?}", session_id, response);
            Ok(HttpResponse::Ok().json(response))
        }
        // **None Pattern** - HashMap::get() returns None when key not found
        None => {
            let status = SessionStatus {
                session_id: session_id.clone(),
                is_valid: false,
                created_at: None,
            };
            let response = ApiResponse::success("Session not found", Some(status));
            info!("Session {} not found. Response: {:?}", session_id, response);
            // **HTTP 404** - Resource not found
            Ok(HttpResponse::NotFound().json(response))
        }
    } // **Match Exhaustiveness** - Rust ensures all possible values are handled
}

/// Removes a session from the system
/// 
/// **Rust Concepts Demonstrated:**
/// - **Mutable Operations** - remove() requires mutable access
/// - **Option Return Values** - remove() returns Option<T>
/// - **Ownership Transfer** - remove() takes ownership of the removed value
/// - **Binary Match** - Simple Some/None pattern matching
async fn unregister_session(
    path: web::Path<String>,
    sessions: web::Data<SessionStore>,
) -> Result<HttpResponse> {
    let session_id = path.into_inner();
    
    info!("Unregistering session: {}", session_id);
    
    // **Mutable Lock** - Need `mut` because we're modifying the HashMap
    let mut sessions_map = sessions.lock().unwrap();
    
    // **HashMap::remove()** - Returns Option<T>, taking ownership of removed item
    match sessions_map.remove(&session_id) {
        // **Ownership Transfer** - remove() gives us ownership of the Session
        Some(session) => {
            let response = ApiResponse::success("Session unregistered successfully", Some(session));
            info!("Session {} unregistered successfully. Response: {:?}", session_id, response);
            Ok(HttpResponse::Ok().json(response))
        }
        // **Not Found Case** - Session didn't exist
        None => {
            // **Unit Type in Generics** - ApiResponse::<()> when no data
            let response = ApiResponse::<()>::error("Session not found");
            info!("Failed to unregister session {} - not found. Response: {:?}", session_id, response);
            Ok(HttpResponse::NotFound().json(response))
        }
    }
}

/// Simple health check endpoint - demonstrates minimal async function
/// 
/// **Rust Concepts Demonstrated:**
/// - **Simple async function** - No complex logic, just returns a response
/// - **String literals** - "Server is running" and "OK" 
/// - **Type inference** - Rust infers types from context
async fn health_check() -> Result<HttpResponse> {
    // **Nested Function Calls** - Building response in one expression
    Ok(HttpResponse::Ok().json(
        ApiResponse::success("Server is running", Some("OK"))
    ))
}


// === CONFIGURATION AND INITIALIZATION SECTION ===

/// Loads application configuration from TOML file
/// 
/// **Rust Concepts Demonstrated:**
/// - **Error Handling** - Result<T, E> for operations that can fail
/// - **Trait Objects** - Box<dyn std::error::Error> for dynamic error types
/// - **? Operator** - Propagates errors up the call stack
/// - **Type Annotations** - Explicit type for disambiguation
/// - **File I/O** - Reading files with error handling
fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // **File Reading** - read_to_string() returns Result<String, io::Error>
    // **? Operator** - If Err, convert to Box<dyn Error> and return early
    let config_content = std::fs::read_to_string("config.toml")?;
    
    // **TOML Parsing** - Convert string to strongly-typed Config struct
    // **Type Annotation** - : Config tells Rust what type to deserialize to
    let config: Config = toml::from_str(&config_content)?;
    
    // **Success Case** - Wrap in Ok() for Result type
    Ok(config)
}

/// Configures the logging system based on configuration
/// 
/// **Rust Concepts Demonstrated:**
/// - **Borrowed Parameters** - &LoggingConfig borrows instead of taking ownership
/// - **Mutable Variables** - `mut builder` for configuration
/// - **Match Expressions** - Pattern matching on string values
/// - **Method Chaining** - Builder pattern for configuration
/// - **Conditional Logic** - if/else for different configurations
/// - **Unit Type** - () represents "no meaningful return value"
fn setup_logging(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    // **Mutable Variable** - Builder pattern requires mutation
    let mut builder = env_logger::Builder::new();
    
    // **String Methods** - to_lowercase() and as_str() for processing
    // **Match on String Slices** - Pattern matching for string values
    let level = match config.log_level.to_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,  // **Default Case** - Fallback pattern
    };
    
    // **Method Chaining** - Configure the builder
    builder.filter_level(level);
    
    // **Conditional Configuration** - Different behavior based on flag
    if config.console_logging {
        builder.init();
    } else {
        // **Complex Boxing** - Box::new() for heap allocation
        // **std::io::sink()** - Discards all writes (like /dev/null)
        builder.target(env_logger::Target::Pipe(Box::new(std::io::sink())));
        builder.init();
    }
    
    // **Unit Return** - () wrapped in Ok for Result<(), E>
    Ok(())
}

// === MAIN APPLICATION ENTRY POINT ===

/// Application entry point - demonstrates async main and server setup
/// 
/// **Rust Concepts Demonstrated:**
/// - **Attribute Macros** - #[actix_web::main] transforms async main
/// - **Async Main Function** - Entry point can be async in some frameworks
/// - **Error Propagation** - Multiple uses of ? operator
/// - **Error Mapping** - Converting between error types
/// - **Variable Shadowing** - Using same names for related values
/// - **Closure Syntax** - move closure for server factory
/// - **Thread Safety** - Arc for sharing data between threads
#[actix_web::main]  // **Attribute Macro** - Transforms async main into sync with runtime
async fn main() -> std::io::Result<()> {
    // **Error Handling Chain** - Load config with error conversion
    let config = load_config().map_err(|e| {
        eprintln!("Failed to load config: {}", e);
        // **Error Type Conversion** - Convert any error to io::Error
        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
    })?;  // **? Operator** - Propagate if error, unwrap if success

    // **Conditional Error Handling** - Different pattern from above
    if let Err(e) = setup_logging(&config.logging) {
        eprintln!("Failed to setup logging: {}", e);
        // **Early Return** - Explicit return with error
        return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
    }

    // **Shared State Initialization** - Thread-safe session storage
    // **Type Annotation** - Explicit type helps with clarity
    let session_store: SessionStore = Arc::new(Mutex::new(HashMap::new()));

    // **Console Output** - User-friendly startup information
    println!("🚀 OneClick BRM Rust Server starting...");
    println!("📡 Server will be available at: http://{}:{}", config.server.host, config.server.port);
    println!("📝 Logging to file: {}", config.logging.log_file);
    println!("🔍 Available endpoints:");
    println!("   GET  /health            - Health check");
    println!("   POST /register/{{id}}     - Register session");
    println!("   GET  /status/{{id}}       - Get session status");
    println!("   DELETE /unregister/{{id}} - Unregister session");
    println!("   POST /obrm/load_obrm_fields/{{id}} - Load Oracle BRM fields");
    println!("   GET  /obrm/get_session_fields/{{id}} - Get loaded BRM fields");
    println!("   POST /obrm/convert_fld_spec_to_podl/{{id}} - Convert field spec to PODL");
    println!("   POST /obrm/convert_class_spec_to_podl/{{id}} - Convert class spec to PODL");
    println!("   POST /obrm/convert_flist2code/{{id}} - Convert FLIST to C code");
    println!("   POST /obrm/convert_flist2xml/{{id}} - Convert FLIST to XML");
    println!("   POST /obrm/convert_flist2json/{{id}} - Convert FLIST to JSON");
    println!("   POST /obrm/view_call_stack/{{id}} - View call stack from ZIP file (returns ZIP with HTML)");

    // **Variable Preparation** - Clone values needed in closure
    let log_file = config.logging.log_file.clone();
    let bind_address = format!("{}:{}", config.server.host, config.server.port);

    // **HTTP Server Setup** - Actix-web server configuration
    HttpServer::new(move || {  // **Move Closure** - Takes ownership of captured variables
        // **CORS Configuration** - Allow cross-origin requests from frontend
        let cors = Cors::default()
            .allowed_origin("http://localhost:8008")  // Dashboard frontend
            .allowed_origin("http://127.0.0.1:8008")  // Alternative localhost
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .max_age(3600);

        App::new()
            // **Dependency Injection** - Make session store available to handlers
            .app_data(web::Data::new(session_store.clone()))
            // **Middleware Stack** - Applied in reverse order (last added = first executed)
            .wrap(cors)                                          // CORS middleware
            .wrap(RequestResponseLogger::new(log_file.clone()))  // Custom middleware
            .wrap(Logger::default())                             // Built-in logging
            // **Route Configuration** - HTTP method + path + handler function
            .route("/health", web::get().to(health_check))
            .route("/register/{session_id}", web::post().to(register_session))
            .route("/status/{session_id}", web::get().to(get_session_status))
            .route("/unregister/{session_id}", web::delete().to(unregister_session))
            // BRM endpoints
            .route("/obrm/load_obrm_fields/{session_id}", web::post().to(load_obrm_fields_handler))
            .route("/obrm/get_session_fields/{session_id}", web::get().to(get_session_fields_handler))
            .route("/obrm/convert_fld_spec_to_podl/{session_id}", web::post().to(convert_fld_spec_to_podl_handler))
            .route("/obrm/convert_class_spec_to_podl/{session_id}", web::post().to(convert_class_spec_to_podl_handler))
            // FLIST conversion endpoints
            .route("/obrm/convert_flist2code/{session_id}", web::post().to(convert_flist2code_handler))
            .route("/obrm/convert_flist2xml/{session_id}", web::post().to(convert_flist2xml_handler))
            .route("/obrm/convert_flist2json/{session_id}", web::post().to(convert_flist2json_handler))
            // Call stack visualization endpoint
            .route("/obrm/view_call_stack", web::post().to(view_call_stack_handler))
    })
    .bind(&bind_address)?     // **Network Binding** - Can fail, hence ?
    .run()                    // **Start Server** - Returns a Future
    .await                    // **Await Completion** - Run until shutdown
}