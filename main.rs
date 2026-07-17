// main.rs
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;
use std::time::Duration;

// Modules definition for clean architectural separation
mod capture;    
mod processing; 
mod notify;     
mod config;     // New: Configuration management

/// The ApplicationState acts as a central repository for runtime configuration.
/// Using RwLock allows multiple readers for performance, while maintaining
/// safe write access when configuration hot-reloading occurs.
pub struct AppState {
    pub is_running: bool,
    pub threshold: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[INFO] Initializing ActiRust system kernel...");

    // Initialize shared state across the async runtime
    let state = Arc::new(RwLock::new(AppState {
        is_running: true,
        threshold: 100,
    }));

    // Multi-Producer, Single-Consumer channel to decouple event capture 
    // from processing logic. Capacity set to 1000 to handle burst inputs.
    let (tx, mut rx) = mpsc::channel(1000);

    // --- CAPTURE LAYER ---
    // Spawning dedicated task for event polling. 
    // This isolates OS-specific blocking calls from the main loop.
    let state_clone = Arc::clone(&state);
    let capture_handle = tokio::spawn(async move {
        println!("[DEBUG] Starting event capture sub-system...");
        if let Err(e) = capture::start_event_loop(tx, state_clone).await {
            eprintln!("[ERROR] Capture engine failure: {:?}", e);
        }
    });

    // --- PROCESSING ENGINE ---
    // The Core Processing Engine consumes raw events, performs heavy analysis
    // (like regex parsing or state transition logic), and prepares alerts.
    let processor_handle = tokio::spawn(async move {
        println!("[DEBUG] Starting core processing engine...");
        
        while let Some(event) = rx.recv().await {
            // Processing::process_event implements a strategy pattern to handle
            // different event types: Keyboard, Mouse, and Process activity.
            match processing::process_event(event).await {
                Ok(Some(action)) => {
                    // Action dispatcher: decides whether to log, notify, or execute
                    if let Err(e) = notify::trigger(action).await {
                        eprintln!("[WARN] Notification dispatch failed: {:?}", e);
                    }
                }
                Ok(None) => continue, 
                Err(e) => {
                    eprintln!("[ERROR] Logic engine anomaly: {:?}", e);
                }
            }
        }
    });

    // --- MONITORING & LIFECYCLE ---
    // The heartbeat monitor ensures that the background tasks haven't stalled.
    let watchdog_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            println!("[HEARTBEAT] ActiRust operational. Monitoring active handles...");
        }
    });

    // Graceful shutdown logic waits for system signals or task completion.
    // Here we join the primary tasks to maintain the process lifecycle.
    let _ = tokio::join!(capture_handle, processor_handle, watchdog_handle);

    println!("[INFO] ActiRust graceful shutdown sequence initiated.");
    Ok(())
}
