//! Event helpers and convenience constructors
//!
//! Re-exports CoordinationEvent convenience constructors from aof-core.
//! The convenience constructors (agent_started, agent_completed, tool_executing, etc.)
//! are implemented on CoordinationEvent in aof-core and are available through this module.

// All convenience constructors are available directly on CoordinationEvent
// from aof-core, so this module serves as a documentation entry point.
pub use aof_core::CoordinationEvent;
