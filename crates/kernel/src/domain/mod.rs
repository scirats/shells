mod name;
mod phase;
mod profile;
mod shell;

pub use name::ShellName;
pub use phase::VmPhase;
pub use profile::{Profile, REQUIRED_PACKAGES};
pub use shell::Shell;

pub const GUEST_STAGE_DIR: &str = "/tmp/shell";
