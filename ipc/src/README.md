# IPC

This crate provides IPC abstractions for communicating between processes on the same machine. On Windows, this implementation uses named pipes with overlapped I/O and I/O completion ports.

All IPC operations are async/await compatible and are implemented in the futures 0.3-preview crate. They should be fairly easy to port to the final std::futures library once that migration completes.