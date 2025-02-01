#[cfg(feature = "tree_visualization")]
use super::Status;
use crate::{factory::FastHashMap, node::BlackBoardCell};
use downcast_rs::{impl_downcast, Downcast};
#[cfg(feature = "expression_node")]
use num::cast::FromPrimitive;
use std::{
    any::TypeId,
    cell::RefCell,
    fmt::Debug,
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub trait BlackBoardType: Downcast + Debug + 'static {
    fn clone_box(&self) -> Box<dyn BlackBoardType>;
}
impl_downcast!(BlackBoardType);
impl<T: Debug + Clone + 'static> BlackBoardType for T {
    fn clone_box(&self) -> Box<dyn BlackBoardType> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn BlackBoardType> {
    fn clone(&self) -> Box<dyn BlackBoardType> {
        self.as_ref().clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct SharedBlackBoardValue {
    bb_name: Rc<String>,
    bb_type: &'static str,
    bb_value: Rc<RefCell<Box<dyn BlackBoardType>>>,
}
impl SharedBlackBoardValue {
    pub fn new(bb_name: String, bb_type: &'static str, bb_value: Box<dyn BlackBoardType>) -> Self {
        Self {
            bb_type,
            bb_name: Rc::new(bb_name),
            bb_value: Rc::new(RefCell::new(bb_value)),
        }
    }

    #[inline]
    pub fn bb_name(&self) -> &str {
        self.bb_name.as_str()
    }

    #[inline]
    pub fn bb_type(&self) -> &'static str {
        self.bb_type
    }

    #[inline]
    pub fn bb_type_id(&self) -> TypeId {
        self.bb_value.borrow().clone().type_id()
    }

    #[inline]
    pub fn borrow(&self) -> std::cell::Ref<Box<dyn BlackBoardType>> {
        self.bb_value.borrow()
    }

    #[inline]
    pub fn borrow_mut(&self) -> std::cell::RefMut<Box<dyn BlackBoardType>> {
        self.bb_value.borrow_mut()
    }

    #[inline]
    #[allow(dead_code)]
    fn get_value<T: BlackBoardType + Clone>(&self) -> Option<T> {
        self.borrow().downcast_ref::<T>().map(|v| v.clone())
    }

    #[inline]
    #[allow(dead_code)]
    fn set_value<T: BlackBoardType>(&self, val: T) -> bool {
        self.borrow_mut()
            .downcast_mut::<T>()
            .map(|v| *v = val)
            .is_some()
    }

    cfg_expression_node! {
        #[inline]
        pub fn is_expr_var(&self) -> bool {
            self.bb_type == "i32"
                || self.bb_type == "i64"
                || self.bb_type == "f32"
                || self.bb_type == "f64"
        }

        pub fn get_as_f64(&self) -> Option<f64> {
            match self.bb_type {
                "i32" => self.get_value::<i32>().map(|v| v as f64),
                "i64" => self.get_value::<i64>().map(|v| v as f64),
                "f32" => self.get_value::<f32>().map(|v| v as f64),
                "f64" => self.get_value::<f64>(),
                _ => None,
            }
        }

        pub fn set_from_f64(&self, val: f64) -> bool {
            match self.bb_type {
                "i32" => i32::from_f64(val).filter(|v| self.set_value(*v)).is_some(),
                "i64" => i64::from_f64(val).filter(|v| self.set_value(*v)).is_some(),
                "f32" => f32::from_f64(val).filter(|v| self.set_value(*v)).is_some(),
                "f64" => f64::from_f64(val).filter(|v| self.set_value(*v)).is_some(),
                _ => false,
            }
        }
    }

    pub fn downcast_to_blackboard_cell<T: BlackBoardType>(&self) -> Option<BlackBoardCell<T>> {
        let mut base = self.borrow_mut();
        let base_mut = base.as_mut();
        match base_mut.downcast_mut::<T>() {
            None => None,
            Some(value) => Some(BlackBoardCell::new(
                self.bb_name.as_ref() as *const String,
                value as *mut T,
            )),
        }
    }
}

cfg_tree_visualization! {
    const MAX_NODE_INDEX: usize = 512;
    const SELF_VISUALIZATION_LEN: usize = (2 * MAX_NODE_INDEX) / (8 * size_of::<u64>());

    #[test]
    fn print_size() {
        println!("MAX_TREE_NODE_NUM: {}", MAX_NODE_INDEX);
        println!("RUNTIME_LIST_LEN: {}", SELF_VISUALIZATION_LEN);
    }


    #[derive(Debug, Clone)]
    pub struct Visualization {
        pub tree_name: String,
        pub tree_index: i32,
        pub tree_depth: i32,
        pub self_visualization: [u64; SELF_VISUALIZATION_LEN],
        pub children_visualization: Vec<Visualization>,
    }

    #[derive(serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode, Debug, Clone)]
    pub struct FlattenedVisualization {
        pub tree_name: String,
        pub tree_index: i32,
        pub tree_depth: i32,
        pub visualizetion: [u64; SELF_VISUALIZATION_LEN],
    }

    impl Visualization {
        fn new(tree_name: String, tree_index: i32, tree_depth: i32) -> Self {
            Self {
                tree_name,
                tree_index,
                tree_depth,
                self_visualization: [0; SELF_VISUALIZATION_LEN],
                children_visualization: Vec::new(),
            }
        }

        #[inline]
        pub fn reset(&mut self) {
            self.self_visualization = [0; SELF_VISUALIZATION_LEN];
            self.children_visualization.clear();
        }

        #[inline]
        fn to_flattened_visualization(&self) -> FlattenedVisualization {
            FlattenedVisualization {
                tree_index: self.tree_index,
                tree_depth: self.tree_depth,
                tree_name: self.tree_name.clone(),
                visualizetion: self.self_visualization,
            }
        }

        #[inline]
        fn preorder_to_flattened_visualization(&self, runtime_info: &mut Vec<FlattenedVisualization>) {
            runtime_info.push(self.to_flattened_visualization());
            for child in &self.children_visualization {
                child.preorder_to_flattened_visualization(runtime_info);
            }
        }

        #[inline]
        fn level_order_to_flattened_visualization(&self, runtime_info: &mut Vec<FlattenedVisualization>) {
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(self);

            while let Some(current_node) = queue.pop_front() {
                runtime_info.push(current_node.to_flattened_visualization());
                for child in &current_node.children_visualization {
                    queue.push_back(child);
                }
            }
        }

        pub fn update_node_status(&mut self, node_index: i32, status: Status) {
            assert!(
                node_index <= MAX_NODE_INDEX as i32,
                "node_index out of range, node_index={}, max_nodex_index={}",
                node_index, MAX_NODE_INDEX
            );
            let state_value = match status {
                Status::Idle => 0,
                Status::Success | Status::Branch(_) => 1,
                Status::Failure => 2,
                Status::Running => 3,
            };

            let zero_based_index = node_index - 1;
            let chunk_index = (zero_based_index / 32) as usize;
            let status_bit_position = zero_based_index % 32;
            let status_mask: u64 = 3 << (status_bit_position * 2);

            let inverted_mask: u64 = !status_mask;

            self.self_visualization[chunk_index] = (self.self_visualization[chunk_index] & inverted_mask)
                | (state_value << (status_bit_position * 2));
        }

        pub fn get_node_status(&self, node_index: i32) -> Status {
            assert!(
                node_index <= MAX_NODE_INDEX as i32,
                "node_index out of range, node_index={}, max_nodex_index={}",
                node_index, MAX_NODE_INDEX
            );

            let zero_based_index = node_index - 1;
            let chunk_index = (zero_based_index / 32) as usize;
            let status_bit_position = zero_based_index % 32;

            let status_mask: u64 = 3 << (status_bit_position * 2);
            let state_value =
                (self.self_visualization[chunk_index] & status_mask) >> (status_bit_position * 2);

            match state_value {
                0 => Status::Idle,
                1 => Status::Success,
                2 => Status::Failure,
                3 => Status::Running,
                _ => Status::Success,
            }
        }
    }
}

pub type BlackBoardMap = FastHashMap<String, SharedBlackBoardValue>;
#[derive(Debug)]
pub struct BlackBoard<T> {
    context: T,
    #[cfg(feature = "tree_visualization")]
    visualization: Visualization,
    blackboard_map: FastHashMap<String, SharedBlackBoardValue>,
}

impl<T> Deref for BlackBoard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.context_ref()
    }
}

impl<T> DerefMut for BlackBoard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.context_mut()
    }
}

impl<T> BlackBoard<T> {
    pub fn new(
        context: T,
        #[allow(unused)] tree_name: String,
        #[allow(unused)] tree_index: i32,
        #[allow(unused)] tree_depth: i32,
        blackboard_map: BlackBoardMap,
    ) -> Self {
        Self {
            context,
            blackboard_map,
            #[cfg(feature = "tree_visualization")]
            visualization: Visualization::new(tree_name, tree_index, tree_depth),
        }
    }

    #[inline]
    pub fn context_ref(&self) -> &T {
        &self.context
    }

    #[inline]
    pub fn context_mut(&mut self) -> &mut T {
        &mut self.context
    }

    #[inline]
    pub fn blackboard_map_ref(&self) -> &BlackBoardMap {
        &self.blackboard_map
    }

    #[inline]
    pub fn blackboard_map_mut(&mut self) -> &mut BlackBoardMap {
        &mut self.blackboard_map
    }

    #[inline]
    pub fn visualize_blackboard_map(&self, s: &mut String) -> std::fmt::Result {
        use std::fmt::Write;
        for value in self.blackboard_map.values() {
            write!(s, "{:?}\n", value)?
        }
        Ok(())
    }

    cfg_tree_visualization! {
        #[inline]
        pub fn tree_name(&self) -> &str {
            &self.visualization.tree_name
        }

        #[inline]
        pub fn tree_index(&self) -> i32 {
            self.visualization.tree_index
        }

        #[inline]
        pub fn tree_depth(&self) -> i32 {
            self.visualization.tree_depth
        }

        #[inline]
        pub fn visualization_ref(&self) -> &Visualization {
            &self.visualization
        }

        #[inline]
        pub fn reset_visualization(&mut self) {
            self.visualization.reset()
        }

        #[inline]
        pub fn get_node_status(&self, node_index: i32) -> Status {
            self.visualization.get_node_status(node_index)
        }

        #[inline]
        pub fn update_node_status(&mut self, node_index: i32, status: Status) {
            self.visualization.update_node_status(node_index, status);
        }

        #[inline]
        pub fn update_children_visualization(&mut self, runtime: Visualization) {
            self.visualization.children_visualization.push(runtime);
        }

        #[inline]
        pub fn preorder_to_flattened_visualization(&self) -> Vec<FlattenedVisualization> {
            let mut runtime_info = Vec::new();
            self.visualization
                .preorder_to_flattened_visualization(&mut runtime_info);
            runtime_info
        }

        #[inline]
        pub fn level_order_to_flattened_visualization(&self) -> Vec<FlattenedVisualization> {
            let mut runtime_info = Vec::new();
            self.visualization
                .level_order_to_flattened_visualization(&mut runtime_info);
            runtime_info
        }

    }
}
