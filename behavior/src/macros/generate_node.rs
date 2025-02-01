#![allow(unused_macros)]

#[macro_export]
macro_rules! generate_node {
    ($tree_name:expr, $tree_index:expr, $tree_depth:expr, $node_name:expr, $node_index: expr ; $bb_map:expr, $meta_map:expr, $bb_ref_map:expr, $dyn_ref_map:expr; $($bb_field:ident),*; $($meta_field:ident),*; $($dyn_field:ident),*; $($node_field:ident),*) => {
        {
            let _ = ($tree_name, $tree_index, $tree_depth, $node_name, $node_index, $bb_map, $meta_map, $bb_ref_map, $dyn_ref_map);
            $(
                let $meta_field = $crate::macros::support::get_metadata($tree_name, $tree_index, $tree_depth, $node_name, $node_index, stringify!($meta_field), $meta_map)?;
            )*
            $(
                let $bb_field = $crate::macros::support::get_blackboard($tree_name, $tree_index, $tree_depth, $node_name, $node_index, stringify!($bb_field), $bb_ref_map, $bb_map)?;
            )*
            $(
                let $dyn_field = $crate::macros::support::get_dynamic($tree_name, $tree_index, $tree_depth, $node_name, $node_index, stringify!($dyn_field), $dyn_ref_map, $bb_map)?;
            )*
            Ok(Box::new(Self {
                base: $crate::macros::support::TreeNodeBase::default(),
                $(
                    $meta_field: $meta_field,
                )*
                $(
                    $bb_field: $bb_field,
                )*
                $(
                    $dyn_field: $dyn_field,
                )*
                $(
                    $node_field: $node_field,
                )*
            }))
        }
    };
}
