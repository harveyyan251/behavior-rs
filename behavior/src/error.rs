use std::fmt::Debug;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct TreeLocation {
    tree_name: String,
    tree_index: i32,
    tree_depth: i32,
}

impl TreeLocation {
    pub fn new(tree_name: &str, tree_index: i32, tree_depth: i32) -> Self {
        Self {
            tree_name: tree_name.to_string(),
            tree_index,
            tree_depth,
        }
    }
}
#[allow(dead_code)]
#[derive(Default)]
pub struct NodeLocation {
    tree_location: TreeLocation,
    node_name: String,
    node_index: i32,
}

impl Debug for NodeLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeLocation")
            .field("tree_name", &self.tree_location.tree_name)
            .field("tree_index", &self.tree_location.tree_index)
            .field("tree_depth", &self.tree_location.tree_depth)
            .field("node_name", &self.node_name)
            .field("node_index", &self.node_index)
            .finish()
    }
}

impl NodeLocation {
    pub fn new(
        tree_name: &str,
        tree_index: i32,
        tree_depth: i32,
        node_name: &str,
        node_index: i32,
    ) -> Self {
        Self {
            tree_location: TreeLocation::new(tree_name, tree_index, tree_depth),
            node_name: node_name.to_string(),
            node_index,
        }
    }
}

#[derive(Debug)]
pub enum BehaviorError {
    UnregisteredTreeNode {
        location: NodeLocation,
    },
    UnregisteredBlackBoardType {
        tree_location: TreeLocation,
        blackboard_name: String,
        blackboard_type: String,
    },
    DowncastFailed {
        borrow_name: String,
        borrow_type: String,
    },
    RegexCapturesFailed {
        location: NodeLocation,
        dynamic_ref_name: String,
        pattern_str: String,
        capture_str: String,
    },
    CompileTreeTemplateFailed {
        tree_name: String,
        error_info: String,
    },
    TreeTemplateNodeNotFound {
        tree_name: String,
        tree_index: i32,
        tree_depth: i32,
    },
    InitBlackBoardParseFailed {
        tree_location: TreeLocation,
        blackboard_name: String,
        blackboard_raw_str: String,
        blackboard_type: String,
    },
    CreateSubTreeFailed {
        node_index: i32,
        parent_name: String,
        subtree_name: String,
        create_error: Box<BehaviorError>,
    },
    MetaDataNotFound {
        location: NodeLocation,
        metadata_name: String,
    },
    MetaDataParseFailed {
        location: NodeLocation,
        metadata_name: String,
        metadata_raw_str: String,
        metadata_type: String,
    },
    BlackBoardRefNotFound {
        location: NodeLocation,
        blackboard_ref_name: String,
    },
    BlackBoardNotFound {
        location: NodeLocation,
        blackboard_ref_name: String,
        blackboard_name: String,
    },
    BlackBoardDowncastFailed {
        location: NodeLocation,
        blackboard_ref_name: String,
        blackboard_name: String,
        blackboard_type: String,
    },
    DynamicRefNotFound {
        location: NodeLocation,
        dynamic_ref_name: String,
    },
    DynamicMetaDataParseFailed {
        location: NodeLocation,
        dynamic_ref_name: String,
        metadata_raw_str: String,
        metadata_type: String,
    },
    DynamicBlackBoardNotFound {
        location: NodeLocation,
        dynamic_ref_name: String,
        blackboard_name: String,
        blackboard_type: String,
    },
    DynamicBlackBoardDowncastFailed {
        location: NodeLocation,
        dynamic_ref_name: String,
        blackboard_name: String,
        blackboard_type: String,
    },
    LinkParentBlackBoardNotFound {
        tree_index: i32,
        tree_depth: i32,
        parent_tree_name: String,
        subtree_name: String,
        parent_blackboard_name: String,
    },
    LinkDifferentBlackBoardType {
        tree_location: TreeLocation,
        parent_name: String,
        subtree_name: String,
        parent_blackboard_name: String,
        subtree_blackboard_name: String,
        parent_blackboard_type: String,
        subtree_blackboard_type: String,
    },
    ExpressionInvalidOperatorTree {
        tree_location: TreeLocation,
        node_index: i32,
        expression: String,
        error_info: String,
    },
    ExpressionInvalidVariable {
        tree_location: TreeLocation,
        node_index: i32,
        expression: String,
        blackboard_name: String,
        blackboard_type: String,
    },
    ExpressionVariableNotExist {
        tree_location: TreeLocation,
        node_index: i32,
        expression: String,
        blackboard_name: String,
    },
}

impl core::fmt::Display for BehaviorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO: Improve this?
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_error() {
        let error = BehaviorError::UnregisteredTreeNode {
            location: NodeLocation::new("test", 0, 0, "test", 0),
        };
        println!("{}", error);
    }
}
