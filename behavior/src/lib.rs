#[macro_use]
pub mod macros;
mod error;
pub use error::{BehaviorError, NodeLocation, TreeLocation};
mod template;
pub use template::TreeNodeStatus;
pub use template::*;
mod instance;
cfg_gen_editor_data! {
    pub use instance::factory::{EditorEnumDataGenerator, EditorNodeDataGenerator};
}
pub use instance::*;
