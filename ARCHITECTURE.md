# Architecture of ActiRust

ActiRust is designed as a high-performance, low-overhead system monitoring tool. The architecture prioritizes minimal resource consumption while maintaining cross-platform compatibility.

## High-Level Overview

The system is built on a modular architecture to separate the event-capturing logic from the analysis and notification engines.

```mermaid.
graph TD
    A[OS Events: Keystrokes, Mouse, Processes] --> B[Event Capture Layer]
    B --> C[Core Processing Engine]
    C --> D[Trigger/Notification System]
    C --> E[Data Logging]
