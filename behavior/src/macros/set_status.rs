cfg_tree_visualization! {
    macro_rules! set_status {
        ($self:ident, $blackboard:ident, $status:expr) => {{
            $self.set_status($status);
            $blackboard.update_node_status($self.index, $status);
            $status
        }};
    }

    macro_rules! set_subtree_status {
        ($self:ident, $blackboard:ident, $status:expr) => {{
            $self.set_status($status);
            $blackboard.update_node_status($self.index, $status);
            $blackboard.update_children_visualization($self.blackboard.visualization_ref().clone());
            $status
        }};
    }
}

cfg_not_tree_visualization! {
    macro_rules! set_status {
        ($self:ident, $blackboard:ident, $status:expr) => {{
            $self.set_status($status);
            $status
        }};
    }

    macro_rules! set_subtree_status {
        ($self:ident, $blackboard:ident, $status:expr) => {{
            $self.set_status($status);
            $status
        }};
    }
}
