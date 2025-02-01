macro_rules! panic_if_idle_or_branch {
    ($node_name:expr, $node_index:expr, $child_name:expr, $child_index:expr) => {
        panic!(
            "{}'s child should never return Idle or Branch status, index={}, child_name={}, child_index={}",
            $node_name, $node_index, $child_name, $child_index
        )
    };
}

macro_rules! panic_if_idle {
    ($node_name:expr, $node_index:expr, $child_name:expr, $child_index:expr) => {
        panic!(
            "{}'s branch condition child should never return Idle, index={}, child_name={}, child_index={}",
            $node_name, $node_index, $child_name, $child_index
        )
    };
}
