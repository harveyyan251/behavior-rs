#![allow(unused_macros)]

macro_rules! cfg_gen_editor_data {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "gen_editor_data")]
            $item
        )*
    }
}

macro_rules! cfg_not_gen_editor_data {
    ($($item:item)*) => {
        $(
            #[cfg(not(feature = "gen_editor_data"))]
            $item
        )*
    }
}

macro_rules! cfg_tree_visualization {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "tree_visualization")]
            $item
        )*
    }
}

macro_rules! cfg_not_tree_visualization {
    ($($item:item)*) => {
        $(
            #[cfg(not(feature = "tree_visualization"))]
            $item
        )*
    }
}

macro_rules! cfg_expression_node {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "expression_node")]
            $item
        )*
    }
}
