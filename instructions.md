I am building a Rust CLI tool. I need to set up rusqlite with the sqlite-vec extension and fastembed-rs to create a 384-dimensional vector table for semantic search. Can you help me write the db.rs module to initialize these tables and the logic to insert a command along with its embedding

🛠️ Phase 1: Storage & Extension Setup
We need to move from JSON to a vector-capable database.

Database: Switch to SQLite using the rusqlite crate.

Vector Extension: Integrate sqlite-vec. This is a lightweight C extension that adds vector math directly to SQLite. In Rust, you can use the sqlite-vec crate to statically link it.

Schema Design:

commands table: id (UUID), cmd, description, created_at.

cmd_embeddings (Virtual Table): Using vec0 from sqlite-vec to store 384-dimensional vectors.

🧠 Phase 2: Local Embedding Engine
Since we aren't using an API, we need a way to run the model in Rust.

Library: Use fastembed-rs. It is a wrapper around ONNX Runtime designed specifically for this. It is extremely fast and handles the all-MiniLM-L6-v2 model automatically.

Logic: * On first boot, the app downloads the 22MB model file to a local cache folder.

Create an Embedder struct that takes a string and returns a Vec<f32> (384 floats).

🔄 Phase 3: The Migration & Ingestion Logic
JSON Import: Write a one-time migration script that reads your old commands.json, generates embeddings for each command, and saves them into the new SQLite DB.

The "Save" Hook: Update your komando --save logic. When a new command is added:

Save to the main table.

Generate an embedding of the command string.

Insert the embedding into the vec0 virtual table.

🔍 Phase 4: Semantic Search Implementation
This is where the magic happens. Your search UI will now:

Take the user's search query (e.g., "delete logs").

Embed the query using fastembed.

Run a K-Nearest Neighbors (KNN) search in SQLite:

SQL
SELECT cmd_id, distance 
FROM cmd_embeddings 
WHERE embedding MATCH ?1 
ORDER BY distance 
LIMIT 10;
Display results ranked by similarity score.

🚀 Phase 5: Shell Integration & UX
Init Command: Implement komando init to provide the eval alias automatically.

Performance: Ensure the UI remains snappy. Since all-MiniLM-L6-v2 is small, the search should take less than 50ms.