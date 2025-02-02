use super::convert::ConvertFromStr;
use super::node::{BtNode, BtNodeGenerator, Executor};
use super::serde::BlackBoardTemplate;
use super::{node::BtAction, serde::TreeTemplate};
use crate::{
    BehaviorError, BlackBoard, BlackBoardMap, BlackBoardType, SharedBlackBoardValue, Status,
    TreeLocation, TreeNode, TreeNodeType,
};

cfg_tree_visualization!(
    pub use crate::{Visualization, FlattenedVisualization};
);
use ahash::HashMapExt;
use behavior_util::type_name;
use std::collections::HashMap;
use std::marker::PhantomPinned;
use std::pin::Pin;

pub type ActionTickFunc<C, W, E> =
    dyn FnMut(&mut BtAction<C, W, E>, &mut BlackBoard<C>, &mut W, &E) -> Status;
pub type TreeState<C, W, E> = TreeNodeType<BtAction<C, W, E>, C, ActionTickFunc<C, W, E>, W, E>;

pub struct BtInstance<C: Unpin + Default + 'static, W: 'static, E: 'static> {
    #[allow(dead_code)]
    pub(crate) tree_name: String,
    pub(crate) tree_state: TreeState<C, W, E>,
    pub(crate) tree_blackboard: BlackBoard<C>,
    _pinned: PhantomPinned,
}

unsafe impl<C: Unpin + Default + 'static, W: 'static, E: 'static> Send for BtInstance<C, W, E> {}
unsafe impl<C: Unpin + Default + 'static, W: 'static, E: 'static> Sync for BtInstance<C, W, E> {}

impl<C: Unpin + Default + 'static, W: 'static, E: 'static> BtInstance<C, W, E> {
    #[inline]
    pub fn tree_name(&self) -> &str {
        &self.tree_name.as_str()
    }

    #[inline]
    pub fn blackboard_map_ref(&self) -> &BlackBoardMap {
        &self.tree_blackboard.blackboard_map_ref()
    }

    #[inline]
    pub fn blackboard_map_mut(self: Pin<&mut Self>) -> Pin<&mut BlackBoardMap> {
        Pin::new(
            unsafe { self.get_unchecked_mut() }
                .tree_blackboard
                .blackboard_map_mut(),
        )
    }

    #[inline]
    pub fn context_ref(&self) -> &C {
        self.tree_blackboard.context_ref()
    }

    #[inline]
    pub fn context_mut(self: Pin<&mut Self>) -> &mut C {
        unsafe { self.get_unchecked_mut() }
            .tree_blackboard
            .context_mut()
    }

    #[inline]
    pub fn visualize_blackboard_map(&self) -> Result<String, std::fmt::Error> {
        let mut buffer = String::new();
        self.tree_blackboard.visualize_blackboard_map(&mut buffer)?;
        Ok(buffer)
    }

    pub fn tick(self: Pin<&mut Self>, world: &mut W, entity: &E) -> Status {
        let BtInstance {
            ref mut tree_state,
            ref mut tree_blackboard,
            ..
        } = unsafe { self.get_unchecked_mut() };

        let mut tick_func =
            |task: &mut BtAction<C, W, E>,
             blackboard: &mut BlackBoard<C>,
             world: &mut W,
             entity: &E|
             -> Status { task.action_tick(blackboard.context_mut(), world, &entity) };

        #[cfg(feature = "tree_visualization")]
        tree_blackboard.reset_visualization();
        tree_state.control_tick(tree_blackboard, &mut tick_func, world, &entity)
    }

    pub fn customized_tick(
        self: Pin<&mut Self>,
        world: &mut W,
        entity: &E,
        tick_func: &mut ActionTickFunc<C, W, E>,
    ) -> Status {
        let BtInstance {
            ref mut tree_state,
            ref mut tree_blackboard,
            ..
        } = unsafe { self.get_unchecked_mut() };

        #[cfg(feature = "tree_visualization")]
        tree_blackboard.reset_visualization();
        tree_state.control_tick(tree_blackboard, tick_func, world, &entity)
    }

    cfg_tree_visualization! {
        #[inline]
        pub fn visualization_ref(&self) -> &Visualization {
            self.tree_blackboard.visualization_ref()
        }

        #[inline]
        pub fn to_flattened_visualization(&self) -> Vec<FlattenedVisualization> {
            self.tree_blackboard
                .level_order_to_flattened_visualization()
        }

        #[inline]
        pub fn visualize_tree_state(&self) -> Result<String, std::fmt::Error> {
            let BtInstance {
                tree_state,
                tree_blackboard,
                ..
            } = self;
            let mut buffer = String::new();
            tree_state.visualize(&mut buffer, tree_blackboard.visualization_ref(), true, 0)?;
            Ok(buffer)
        }
    }
}

/// link blackboard from parent tree to child tree
pub(crate) struct ParentTreeLink<'parent> {
    pub(crate) parent_tree_name: &'parent str,
    pub(crate) parent_blackboard_map: &'parent BlackBoardMap,
    pub(crate) parent_blackboard_ref_map: &'parent HashMap<String, String>,
}

impl<'parent> ParentTreeLink<'parent> {
    pub fn new(
        parent_tree_name: &'parent str,
        parent_bb_map: &'parent BlackBoardMap,
        parent_ref_map: &'parent HashMap<String, String>,
    ) -> Self {
        Self {
            parent_tree_name,
            parent_blackboard_map: parent_bb_map,
            parent_blackboard_ref_map: parent_ref_map,
        }
    }
}

cfg_gen_editor_data! {
    pub trait EditorNodeDataGenerator {
        fn generate_editor_node_data() -> serde_json::Value;
    }

    pub trait EditorEnumDataGenerator {
        fn generate_editor_enum_data() -> serde_json::Value;
    }

    type EditorNodeDataGenerateFunc = fn() -> serde_json::Value;
    type EditorEnumDataGenerateFunc = fn() -> serde_json::Value;
    pub struct GenEditorData {
        enum_data_generator: Vec<EditorNodeDataGenerateFunc>,
        node_data_generator: Vec<EditorEnumDataGenerateFunc>,
    }

    impl GenEditorData {
        pub fn new() -> Self {
            Self {
                enum_data_generator: Vec::new(),
                node_data_generator: Vec::new(),
            }
        }
        cfg_gen_editor_data! {
            pub fn push_enum_data_generator(&mut self, generator: EditorNodeDataGenerateFunc) {
                self.enum_data_generator.push(generator);
            }

            pub fn push_node_data_generator(&mut self, generator: EditorEnumDataGenerateFunc) {
                self.node_data_generator.push(generator);
            }
        }
    }
}

pub(crate) type FastHashMap<K, V> = HashMap<K, V, ahash::RandomState>;

type NodeGenerateFunc<C, W, E> = fn(
    tree_name: &str,
    tree_index: i32,
    tree_depth: i32,
    node_name: &str,
    node_index: i32,
    blackboard_map: &BlackBoardMap,
    metadata_map: Option<&HashMap<String, String>>,
    blackboard_ref_map: Option<&HashMap<String, String>>,
    dynamic_ref_map: Option<&HashMap<String, String>>,
) -> Result<Executor<C, W, E>, BehaviorError>;

type InitBlackboardFunc = fn(
    tree_name: &str,
    tree_index: i32,
    tree_depth: i32,
    parent_tree_name: &str,
    blackboard_map: &mut BlackBoardMap,
    blackboard_template: &BlackBoardTemplate,
) -> Result<(), BehaviorError>;

pub struct BtFactory<C: Default + 'static, W: 'static, E: 'static> {
    tree_template_map: FastHashMap<String, TreeTemplate>,
    node_generator_map: FastHashMap<String, NodeGenerateFunc<C, W, E>>,
    init_blackboard_map: FastHashMap<&'static str, InitBlackboardFunc>,
    #[cfg(feature = "gen_editor_data")]
    gen_editor_data: GenEditorData,
}

// Only use in lib crate
impl<C: Unpin + Default + 'static, W: 'static, E: 'static> BtFactory<C, W, E> {
    pub(crate) fn inner_create_tree_instance(
        &self,
        tree_name: &str,
        tree_index: i32,
        tree_depth: i32,
        parent_tree_link: Option<ParentTreeLink>,
    ) -> Result<BtInstance<C, W, E>, BehaviorError> {
        match self.tree_template_map.get(tree_name) {
            None => Err(BehaviorError::TreeTemplateNodeNotFound {
                tree_name: tree_name.to_string(),
                tree_index,
                tree_depth,
            }),
            Some(tree_template) => {
                let parent_link_info = match parent_tree_link {
                    None => None,
                    Some(parent_tree_link) => {
                        let ParentTreeLink {
                            parent_tree_name,
                            parent_blackboard_map,
                            parent_blackboard_ref_map,
                        } = parent_tree_link;

                        let mut subtree_blackboard_map = BlackBoardMap::new();
                        for (subtree_blackboard_name, parent_blackboard_name) in
                            parent_blackboard_ref_map.iter()
                        {
                            let parent_blackboard = parent_blackboard_map
                                .get(parent_blackboard_name)
                                .ok_or_else(|| BehaviorError::LinkParentBlackBoardNotFound {
                                    tree_index,
                                    tree_depth,
                                    subtree_name: tree_name.to_string(),
                                    parent_tree_name: parent_tree_name.to_string(),
                                    parent_blackboard_name: parent_blackboard_name.to_string(),
                                })?;
                            subtree_blackboard_map.insert(
                                subtree_blackboard_name.to_string(),
                                parent_blackboard.clone(),
                            );
                        }
                        Some((subtree_blackboard_map, parent_tree_name))
                    }
                };
                let (tree_blackboard, tree_state) = tree_template.to_instance(
                    &self,
                    tree_name,
                    tree_index,
                    tree_depth,
                    parent_link_info,
                )?;
                Ok(BtInstance {
                    tree_name: tree_name.to_string(),
                    tree_state,
                    tree_blackboard,
                    _pinned: PhantomPinned,
                })
            }
        }
    }

    fn inner_init_blackboard<T: BlackBoardType + ConvertFromStr>(
        tree_name: &str,
        tree_index: i32,
        tree_depth: i32,
        parent_tree_name: &str,
        blackboard_map: &mut BlackBoardMap,
        blackboard_template: &BlackBoardTemplate,
    ) -> Result<(), BehaviorError> {
        let blackboard_name = blackboard_template.bb_name.as_str();
        let blackboard_raw_str = blackboard_template.bb_value.as_str();
        let blackboard = match T::convert_from_str(blackboard_raw_str) {
            Some(blackboard) => SharedBlackBoardValue::new(
                blackboard_name.to_string(),
                type_name::<T>(),
                Box::new(blackboard),
            ),
            None => {
                return Err(BehaviorError::InitBlackBoardParseFailed {
                    tree_location: TreeLocation::new(tree_name, tree_index, tree_depth),
                    blackboard_name: blackboard_name.to_string(),
                    blackboard_raw_str: blackboard_raw_str.to_string(),
                    blackboard_type: type_name::<T>().to_string(),
                })
            }
        };

        match blackboard_map.get(blackboard_name) {
            None => {
                blackboard_map.insert(blackboard_name.to_string(), blackboard);
            }
            Some(parent_blackboard) => {
                let parent_type_id = parent_blackboard.bb_type_id();
                let subtree_type_id = blackboard.bb_type_id();
                // TODO: 改为 type_name 比较
                if parent_type_id != subtree_type_id {
                    return Err(BehaviorError::LinkDifferentBlackBoardType {
                        tree_location: TreeLocation::new(tree_name, tree_index, tree_depth),
                        parent_name: parent_tree_name.to_string(),
                        subtree_name: tree_name.to_string(),
                        parent_blackboard_name: parent_blackboard.bb_name().to_string(),
                        subtree_blackboard_name: blackboard_name.to_string(),
                        parent_blackboard_type: parent_blackboard.bb_type().to_string(),
                        subtree_blackboard_type: blackboard.bb_type().to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    pub(crate) fn init_blackboard(
        &self,
        tree_name: &str,
        tree_index: i32,
        tree_depth: i32,
        parent_tree_name: &str,
        blackboard_map: &mut BlackBoardMap,
        blackboard_template: &BlackBoardTemplate,
    ) -> Result<(), BehaviorError> {
        let blackboard_type = blackboard_template.bb_type.as_str();
        self.init_blackboard_map
            .get(blackboard_type)
            .ok_or_else(|| BehaviorError::UnregisteredBlackBoardType {
                tree_location: TreeLocation::new(tree_name, tree_index, tree_depth),
                blackboard_name: blackboard_template.bb_name.to_string(),
                blackboard_type: blackboard_type.to_string(),
            })?(
            tree_name,
            tree_index,
            tree_depth,
            parent_tree_name,
            blackboard_map,
            blackboard_template,
        )
    }

    pub(crate) fn get_node_executor(
        &self,
        tree_name: &str,
        tree_index: i32,
        tree_depth: i32,
        node_name: &str,
        node_index: i32,
        blackboard_map: &BlackBoardMap,
        metadata_map: Option<&HashMap<String, String>>,
        blackboard_ref_map: Option<&HashMap<String, String>>,
        dynamic_ref_map: Option<&HashMap<String, String>>,
    ) -> Result<Executor<C, W, E>, BehaviorError> {
        self.node_generator_map.get(node_name).ok_or_else(|| {
            BehaviorError::UnregisteredTreeNode {
                location: crate::NodeLocation::new(
                    tree_name, tree_index, tree_depth, node_name, node_index,
                ),
            }
        })?(
            tree_name,
            tree_index,
            tree_depth,
            node_name,
            node_index,
            blackboard_map,
            metadata_map,
            blackboard_ref_map,
            dynamic_ref_map,
        )
    }
}

impl<C: Unpin + Default + 'static, W: 'static, E: 'static> BtFactory<C, W, E> {
    pub fn new() -> Self {
        let mut object = Self {
            tree_template_map: FastHashMap::new(),
            node_generator_map: FastHashMap::new(),
            init_blackboard_map: FastHashMap::new(),
            #[cfg(feature = "gen_editor_data")]
            gen_editor_data: GenEditorData::new(),
        };
        // TODO: 简化写法
        object.register_blackboard_type::<bool>();
        object.register_blackboard_type::<i32>();
        object.register_blackboard_type::<i64>();
        object.register_blackboard_type::<f32>();
        object.register_blackboard_type::<f64>();
        object
    }

    pub fn register_blackboard_type<T: BlackBoardType + ConvertFromStr>(&mut self) {
        self.init_blackboard_map.insert(
            type_name::<T>(),
            Self::inner_init_blackboard::<T> as InitBlackboardFunc,
        );
    }

    cfg_not_gen_editor_data! {
        pub fn register_tree_node<T: BtNode + BtNodeGenerator<Context = C, World = W, Entity = E>>(
            &mut self,
        ) {
            self.node_generator_map
                .insert(type_name::<T>().to_string(), T::generate_node);
        }
    }

    cfg_gen_editor_data! {
        pub fn register_tree_node<
            T: BtNode + BtNodeGenerator<Context = C, World = W, Entity = E> + EditorNodeDataGenerator,
        >(
            &mut self,
        ) {
            self.gen_editor_data
                .push_node_data_generator(T::generate_editor_node_data);
            self.node_generator_map
                .insert(type_name::<T>().to_string(), T::generate_node);
        }

        pub fn register_tree_enum<
            T:  EditorEnumDataGenerator,
        >(
            &mut self,
        ) {
            self.gen_editor_data
                .push_enum_data_generator(T::generate_editor_enum_data);
        }

        // TODO: GenEditorDataFunc
    }

    // TODO: 从文件序列化 json 文件或 bytes 文件, 用 bincode 序列化
    // TODO: 导出编辑器数据到指定文件

    pub fn compile_tree_template_from_json_str(
        &mut self,
        tree_name: &str,
        tree_json_str: &str,
    ) -> Result<(), BehaviorError> {
        match serde_json::from_str(tree_json_str) {
            Ok(tree_template) => {
                self.tree_template_map
                    .insert(tree_name.to_string(), tree_template);
                Ok(())
            }
            Err(e) => Err(BehaviorError::CompileTreeTemplateFailed {
                tree_name: tree_name.to_string(),
                error_info: e.to_string(),
            }),
        }
    }

    pub fn compile_tree_template_from_json_file<P: AsRef<std::path::Path>>(
        &mut self,
        tree_name: &str,
        tree_json_file_path: P,
    ) -> Result<(), BehaviorError> {
        let json_file = match std::fs::File::open(tree_json_file_path) {
            Ok(file) => file,
            Err(error) => {
                return Err(BehaviorError::CompileTreeTemplateFailed {
                    tree_name: tree_name.to_string(),
                    error_info: error.to_string(),
                })
            }
        };

        let reader = std::io::BufReader::new(json_file);
        match serde_json::from_reader(reader) {
            Ok(tree_template) => {
                self.tree_template_map
                    .insert(tree_name.to_string(), tree_template);
                Ok(())
            }
            Err(e) => Err(BehaviorError::CompileTreeTemplateFailed {
                tree_name: tree_name.to_string(),
                error_info: e.to_string(),
            }),
        }
    }

    // TODO: add feature bincode
    // TODO: compile_tree_template_from_bincode_file

    pub fn create_tree_instance(
        &self,
        tree_name: &str,
    ) -> Result<Pin<Box<BtInstance<C, W, E>>>, BehaviorError> {
        static ROOT_TREE_INDEX: i32 = 0;
        static ROOT_TREE_DEPTH: i32 = 0;
        self.inner_create_tree_instance(tree_name, ROOT_TREE_INDEX, ROOT_TREE_DEPTH, None)
            .map(|instance| Box::pin(instance))
    }
}
