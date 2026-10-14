# ArcFlow

**A native Rust node editor for branching narratives and interactive dialogue.**

ArcFlow is a high-performance, desktop-first visual scripting tool designed for game developers and narrative designers. Built with Rust and [egui](https://github.com/emilk/egui), it provides a node-based workspace for designing dialogue trees, defining conditional logic, managing game-state variables, and exporting structured JSON for integration with Unreal Engine and Godot.

Designed around a library-first architecture, ArcFlow separates narrative logic from UI rendering, enabling testable core systems and a foundation for efficient graph processing.

## Table of Contents

- [Why ArcFlow?](#why-arcflow)
- [Key Features](#key-features)
- [Performance Goals](#performance-goals)
- [Architecture](#architecture)
- [How It Works](#how-it-works)
- [Technology Stack](#technology-stack)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

## Why ArcFlow?

Non-linear dialogue is fundamentally a graph problem, yet narrative designers often have to choose between linear writing tools, code-centric scripting systems, and expensive specialized software.

| Tool | Limitation |
|---|---|
| [Twine](https://twinery.org/) | Primarily focused on interactive storytelling, with a workflow that may not suit complex game-engine integrations. |
| [Ink](https://www.inklestudios.com/ink/) | Powerful narrative scripting, but its primary workflow is text-based rather than a fully visual graph editor. |
| [Articy:draft](https://www.articy.com/) | Comprehensive narrative-design capabilities, but its licensing and workflow may not suit every independent developer. |

**ArcFlow aims to bridge this gap** with a lightweight native application that combines visual graph authoring, state-dependent branching, and engine-oriented data export without the overhead of an embedded web runtime.

## Key Features

### Visual Narrative Editor
- Node-based workspace for building branching dialogue.
- Connected graph representation of narrative flow.
- Dedicated canvas, asset browser, and property inspector.
- Dark, minimal interface designed for focused authoring.

### State-Driven Dialogue
- Dialogue nodes for character speech and narrative text.
- Condition nodes for evaluating game-state variables.
- Action nodes for modifying state during execution.
- A centralized blackboard for variables and character definitions.

### Graph Validation
- Directed graph representation of narrative dependencies.
- Detection of unreachable nodes and dead-end paths.
- Cycle analysis to identify potentially unintended dialogue loops.
- Validation before exporting narrative data.

### Engine-Oriented Export
- JSON serialization for downstream game-engine integration.
- Separation of execution logic from presentation metadata.
- Localization string extraction for translation and voice-over workflows.
- An architecture designed to support Unreal Engine and Godot export pipelines.

### Native Performance
- Compiled Rust application with no mandatory Electron runtime.
- Immediate-mode UI rendering using egui.
- Separation of UI state and core narrative data.
- A design intended to support large dialogue graphs efficiently.

## Performance Goals

ArcFlow is designed to minimize startup overhead, memory consumption, and interaction latency compared with conventional web-based node editors.

The following values are **target or illustrative design benchmarks, not independently verified measurements**. Actual performance depends on hardware, graph complexity, rendering configuration, and implementation details.

| Metric | ArcFlow Target | Typical Web-Based Desktop Editor |
|---|---|---|
| Startup time | Under 100 ms | Seconds, depending on the application |
| Memory footprint | Approximately 15–30 MB | Potentially hundreds of MB |
| Rendering | Target 60–144 FPS | Depends on rendering architecture and workload |
| Large graph handling | Target 10,000+ nodes | Depends on virtualization and graph implementation |
| Release binary | Target under 10 MB | Often larger when bundled with a web runtime |

Performance targets should be validated through repeatable benchmarks before being treated as production guarantees.

## Architecture

ArcFlow follows a library-first architecture with three primary layers:

1. **Presentation layer** — renders the editor and translates user interactions into model updates.
2. **Core layer** — owns the narrative graph, project data, and game-state definitions.
3. **Export layer** — validates and transforms project data into engine-consumable representations.

This separation keeps narrative processing independent of the graphical interface and makes the core model suitable for headless unit testing.

### 1. Presentation Layer — `src/ui/`

Built with `egui` and a node-editor component such as `egui_node_graph`.

Responsibilities:
- Render the node canvas and graph connections.
- Display project assets and global variables.
- Provide an inspector for editing selected nodes.
- Dispatch user edits to the core model.
- Manage navigation between the dashboard and editor.

The UI acts as a presentation layer rather than the authoritative owner of narrative logic.

### 2. Core Logic — `src/core/`

Built around Rust data structures and [petgraph](https://github.com/petgraph/petgraph).

Responsibilities:
- Represent dialogue and branching logic as a directed graph.
- Store characters, variables, and project metadata.
- Validate node connectivity and references.
- Detect cycles, unreachable nodes, and potential dead ends.
- Serialize project data for persistence.

The core model serves as the source of truth for the narrative.

### 3. Export Layer — `src/export/`

Built around [Serde](https://serde.rs/) for serialization and structured data transformation.

Responsibilities:
- Validate the graph before compilation.
- Traverse the narrative graph and produce an engine-oriented representation.
- Separate execution data from editor-only metadata.
- Generate JSON output for downstream runtime integration.
- Extract localization keys and translatable strings.

Engine-specific output formats should be implemented through dedicated exporters so that the core model remains independent of any single game engine.

## How It Works

### 1. Define Characters and Variables

Create character definitions and declare global blackboard variables.

Example variables:

| Variable | Type | Example |
|---|---|---|
| `has_sword` | Boolean | `true` |
| `reputation` | Integer | `75` |
| `quest_completed` | Boolean | `false` |

These definitions provide a consistent reference for dialogue conditions and actions.

### 2. Design the Narrative Graph

Build the dialogue flow by connecting nodes on the canvas.

| Node Type | Purpose |
|---|---|
| Dialogue | Defines a speaker and their dialogue text. |
| Condition | Evaluates a variable or expression and branches according to the result. |
| Action | Updates variables or performs a defined state mutation. |

For example, a character might offer a special dialogue option only when `has_sword == true`.

### 3. Inspect and Edit Nodes

Select a node to open its properties in the inspector.

The inspector exposes the fields relevant to that node type, including dialogue text, speaker selection, condition expressions, and action parameters. Character and variable selectors can be populated from the project's registries.

### 4. Validate the Graph

Before export, analyze the graph for structural and logical problems, including missing references, unreachable content, dead ends, and suspicious cycles.

Validation rules should distinguish intentional loops from potentially accidental infinite dialogue paths.

### 5. Export the Project

Transform the validated graph into structured JSON.

The exported representation should contain the information required by a game-side dialogue interpreter without depending on editor-specific node coordinates or presentation metadata.

Localization output can be generated separately to support translation and voice-over production.

## Technology Stack

| Technology | Role |
|---|---|
| Rust | Application implementation and native compilation |
| Cargo | Dependency management, building, and testing |
| egui | Immediate-mode graphical user interface |
| eframe | Native desktop application framework |
| egui_node_graph | Node-editor interface components |
| petgraph | Directed graph representation and traversal |
| Serde | Data serialization and deserialization |
| JSON | Project interchange and export format |

The exact dependencies and versions are defined in `Cargo.toml`.

## Getting Started

### Prerequisites

Install the following tools before building ArcFlow:

- [Rust toolchain](https://rustup.rs/) — includes `rustc` and Cargo.
- [Git](https://git-scm.com/downloads) — for cloning the repository.

On Windows, install the Microsoft C++ Build Tools if your selected Rust dependencies require the MSVC toolchain.

Verify the installation:

```bash
rustc --version
cargo --version
git --version
```

### Clone the Repository

Replace `yourusername` with the actual GitHub account or organization hosting the repository.

```bash
git clone https://github.com/yourusername/arcflow.git
cd arcflow
```

### Run in Development Mode

```bash
cargo run
```

Cargo will resolve dependencies, compile the application, and launch it if the current implementation is complete.

### Build an Optimized Release

```bash
cargo build --release
```

The resulting executable is typically located at:

- **Windows:** `target/release/arcflow.exe`
- **Linux:** `target/release/arcflow`
- **macOS:** `target/release/arcflow`

The actual executable name follows the package configuration in `Cargo.toml`.

### Run Tests

```bash
cargo test
```

Use this command to run the project's Rust unit tests and integration tests.

### Check Code Quality

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

These commands check formatting and run Clippy's lint analysis.

## Project Structure

```text
arcflow/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
└── src/
    ├── main.rs                  # Application entry point
    ├── lib.rs                   # Core library root
    │
    ├── core/                    # Headless narrative model
    │   ├── project.rs           # Project model and persistence
    │   ├── graph.rs             # Directed narrative graph
    │   └── blackboard.rs        # Variables and character registry
    │
    ├── ui/                      # egui presentation layer
    │   ├── app.rs               # Application state and navigation
    │   ├── theme.rs             # UI theme and styling
    │   └── editor/              # Graph editor workspace
    │
    └── export/                  # Validation and compilation
        ├── engine_json.rs       # Engine-oriented JSON generation
        └── localization.rs      # Localization string extraction
```

This tree describes the intended organization; individual modules may be introduced as development progresses.

## Roadmap

The following roadmap outlines potential development milestones.

- [x] Establish the native application shell and project dashboard.
- [ ] Implement project creation, loading, and saving.
- [ ] Integrate the interactive node canvas.
- [ ] Implement dialogue, condition, and action nodes.
- [ ] Add character definitions and blackboard variable management.
- [ ] Implement graph validation and cycle analysis.
- [ ] Build deterministic JSON export.
- [ ] Add localization extraction and string-key management.
- [ ] Implement and test Unreal Engine integration.
- [ ] Implement and test Godot integration.
- [ ] Benchmark large graphs and optimize rendering.
- [ ] Add automated tests, CI, and release builds.

## Contributing

Contributions, bug reports, and feature suggestions are welcome.

To get started:

1. Fork the repository.
2. Create a feature branch.
3. Make your changes and add appropriate tests.
4. Run `cargo fmt --check`, `cargo clippy`, and `cargo test`.
5. Submit a pull request describing the changes and their motivation.

For substantial changes, consider opening an issue first to discuss the proposed design.

## License

ArcFlow is free and open-source software distributed under the [MIT License](file:///home/kaddulive/Desktop/Projects/ArcFlow/LICENSE).


---

**ArcFlow — Design the narrative. Define the logic. Ship the story.**
