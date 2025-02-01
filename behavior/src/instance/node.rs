use super::{convert::ConvertFromStr, factory::ActionTickFunc};
use crate::{
    BehaviorError, BlackBoard, BlackBoardMap, BlackBoardType, NodeLocation, NodeType, Status,
    TreeNode, TreeNodeStatus,
};
use regex::Regex;
use std::{
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::OnceLock,
};

pub trait FromI32: Sized {
    fn from_i32(value: i32) -> Option<Self>;
}

#[derive(Debug)]
pub enum EmptyEnum {}
impl FromI32 for EmptyEnum {
    fn from_i32(_value: i32) -> Option<Self> {
        None
    }
}

#[derive(Debug)]
pub struct MetaDataCell<T, Enum: FromI32 = EmptyEnum> {
    value: T,
    _marker: PhantomData<Enum>,
}
impl<T, Enum: FromI32> MetaDataCell<T, Enum> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }
    pub fn as_ref(&self) -> &T {
        &self.value
    }
}
impl<T, Enum: FromI32> Deref for MetaDataCell<T, Enum> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl<T: Clone, Enum: FromI32> MetaDataCell<T, Enum> {
    pub fn get(&self) -> T {
        self.value.clone()
    }
}

#[derive(Debug)]
pub struct BlackBoardCell<T> {
    name: NonNull<String>,
    value: NonNull<T>,
}
impl<T> BlackBoardCell<T> {
    #[inline]
    pub fn new(name: *const String, value: *mut T) -> Self {
        let name = unsafe { NonNull::new_unchecked(name as *mut String) };
        let value = unsafe { NonNull::new_unchecked(value) };
        Self { name, value }
    }
    #[inline]
    pub fn as_ref(&self) -> &T {
        unsafe { self.value.as_ref() }
    }
    #[inline]
    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.value.as_mut() }
    }
    #[inline]
    pub fn name(&self) -> &str {
        unsafe { self.name.as_ref() }
    }
    #[inline]
    pub fn set(&mut self, value: T) {
        unsafe {
            *self.value.as_mut() = value;
        }
    }
}
impl<T: Clone> BlackBoardCell<T> {
    pub fn get(&self) -> T {
        self.as_ref().clone()
    }
}
impl<T> Deref for BlackBoardCell<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl<T> DerefMut for BlackBoardCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
impl<U> BlackBoardCell<Option<U>> {
    #[inline]
    pub fn is_some(&self) -> bool {
        self.as_ref().is_some()
    }
    #[inline]
    pub fn is_none(&self) -> bool {
        self.as_ref().is_none()
    }
    #[inline]
    pub fn unwrap_ref(&self) -> &U {
        self.as_ref().as_ref().unwrap()
    }
    #[inline]
    pub fn unwrap_mut(&mut self) -> &mut U {
        self.as_mut().as_mut().unwrap()
    }
}
impl<U: Clone> BlackBoardCell<Option<U>> {
    pub fn unwrap_value(&self) -> U {
        self.as_ref().as_ref().unwrap().clone()
    }
}

#[derive(Debug)]
pub struct DummyMutable<T: Clone> {
    pub value: T,
    pub dummy: T,
}
impl<T: Clone> DummyMutable<T> {
    fn new(value: T) -> Self {
        let dummy = value.clone();
        Self { value, dummy }
    }
    #[inline]
    fn get(&self) -> T {
        self.value.clone()
    }
    #[inline]
    fn as_ref(&self) -> &T {
        &self.value
    }
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.dummy = self.get();
        &mut self.dummy
    }
}

#[derive(Debug)]
pub enum DynamicCell<T: Clone> {
    Mutable(BlackBoardCell<T>),
    Immutable(DummyMutable<T>),
}
impl<T: Clone> From<BlackBoardCell<T>> for DynamicCell<T> {
    fn from(value: BlackBoardCell<T>) -> Self {
        Self::Mutable(value)
    }
}
impl<T: Clone> From<DummyMutable<T>> for DynamicCell<T> {
    fn from(value: DummyMutable<T>) -> Self {
        Self::Immutable(value)
    }
}
impl<T: Clone> DynamicCell<T> {
    #[inline]
    pub fn new(data: impl Into<Self>) -> Self {
        data.into()
    }
    #[inline]
    pub fn is_mutable(&self) -> bool {
        matches!(self, DynamicCell::Mutable(_))
    }
    #[inline]
    pub fn as_ref(&self) -> &T {
        match self {
            DynamicCell::Mutable(cell) => cell.as_ref(),
            DynamicCell::Immutable(cell) => cell.as_ref(),
        }
    }
    #[inline]
    pub fn as_mut(&mut self) -> &mut T {
        match self {
            DynamicCell::Mutable(cell) => cell.as_mut(),
            DynamicCell::Immutable(cell) => cell.as_mut(),
        }
    }
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            DynamicCell::Mutable(cell) => cell.name(),
            DynamicCell::Immutable(_) => &"",
        }
    }
    #[inline]
    pub fn set(&mut self, value: T) {
        match self {
            DynamicCell::Mutable(cell) => cell.set(value),
            _ => {}
        }
    }
    #[inline]
    pub fn get(&self) -> T {
        match self {
            DynamicCell::Mutable(cell) => cell.get(),
            DynamicCell::Immutable(cell) => cell.get(),
        }
    }
}
impl<T: Clone> Deref for DynamicCell<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl<T: Clone> DerefMut for DynamicCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
impl<T: Clone> DynamicCell<Option<T>> {
    #[inline]
    pub fn is_some(&self) -> bool {
        self.as_ref().is_some()
    }
    #[inline]
    pub fn is_none(&self) -> bool {
        self.as_ref().is_none()
    }
    #[inline]
    pub fn unwrap_ref(&self) -> &T {
        self.as_ref().as_ref().unwrap()
    }
    #[inline]
    pub fn unwrap_mut(&mut self) -> &mut T {
        self.as_mut().as_mut().unwrap()
    }
    #[inline]
    pub fn unwrap_value(&self) -> T {
        self.as_ref().as_ref().unwrap().clone()
    }
}

pub fn get_metadata<T, Enum: FromI32>(
    tree_name: &str,
    tree_index: i32,
    tree_depth: i32,
    node_name: &str,
    node_index: i32,
    meta_name: &str,
    meta_map: Option<&HashMap<String, String>>,
) -> Result<MetaDataCell<T, Enum>, BehaviorError>
where
    T: BlackBoardType + ConvertFromStr,
{
    let meta_raw_str = meta_map.and_then(|map| map.get(meta_name)).ok_or_else(|| {
        BehaviorError::MetaDataNotFound {
            location: NodeLocation::new(tree_name, tree_index, tree_depth, node_name, node_index),
            metadata_name: meta_name.to_string(),
        }
    })?;

    match T::convert_from_str(meta_raw_str) {
        None => Err(BehaviorError::MetaDataParseFailed {
            location: NodeLocation::new(tree_name, tree_index, tree_depth, node_name, node_index),
            metadata_name: meta_name.to_string(),
            metadata_raw_str: meta_raw_str.to_string(),
            metadata_type: behavior_util::type_name::<T>().to_string(),
        }),
        Some(parsed) => Ok(MetaDataCell::<T, Enum>::new(parsed)),
    }
}

pub fn get_blackboard<T: BlackBoardType>(
    tree_name: &str,
    tree_index: i32,
    tree_depth: i32,
    node_name: &str,
    node_index: i32,
    blackboard_ref_name: &str,
    blackboard_ref_map: Option<&HashMap<String, String>>,
    blackboard_map: &BlackBoardMap,
) -> Result<BlackBoardCell<T>, BehaviorError> {
    let blackboard_name = blackboard_ref_map
        .and_then(|map| map.get(blackboard_ref_name))
        .ok_or_else(|| BehaviorError::BlackBoardRefNotFound {
            location: NodeLocation::new(tree_name, tree_index, tree_depth, node_name, node_index),
            blackboard_ref_name: blackboard_ref_name.to_string(),
        })?;

    match blackboard_map.get(blackboard_name) {
        None => Err(BehaviorError::BlackBoardNotFound {
            location: NodeLocation::new(tree_name, tree_index, tree_depth, node_name, node_index),
            blackboard_ref_name: blackboard_ref_name.to_string(),
            blackboard_name: blackboard_name.to_string(),
        }),
        Some(bb_value) => bb_value.downcast_to_blackboard_cell::<T>().ok_or_else(|| {
            BehaviorError::BlackBoardDowncastFailed {
                location: NodeLocation::new(
                    tree_name, tree_index, tree_depth, node_name, node_index,
                ),
                blackboard_ref_name: blackboard_ref_name.to_string(),
                blackboard_name: blackboard_name.to_string(),
                blackboard_type: behavior_util::type_name::<T>().to_string(),
            }
        }),
    }
}

pub fn get_dynamic<T: BlackBoardType + ConvertFromStr + Clone>(
    tree_name: &str,
    tree_index: i32,
    tree_depth: i32,
    node_name: &str,
    node_index: i32,
    dynamic_ref_name: &str,
    dynamic_ref_map: Option<&HashMap<String, String>>,
    blackboard_map: &BlackBoardMap,
) -> Result<DynamicCell<T>, BehaviorError> {
    let dynamic_raw_str = dynamic_ref_map
        .and_then(|map| map.get(dynamic_ref_name))
        .ok_or_else(|| BehaviorError::DynamicRefNotFound {
            location: NodeLocation::new(tree_name, tree_index, tree_depth, node_name, node_index),
            dynamic_ref_name: dynamic_ref_name.to_string(),
        })?;

    // TODO: 自定义正则表达式
    static PATTERN: &'static str = r"^<(.*?)>$";
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = REGEX.get_or_init(|| match Regex::new(PATTERN) {
        Ok(regex) => regex,
        Err(e) => {
            panic!("Regex init failed!, error={:?}", e);
        }
    });

    match regex.captures(dynamic_raw_str) {
        None => match T::convert_from_str(dynamic_raw_str) {
            None => Err(BehaviorError::DynamicMetaDataParseFailed {
                location: NodeLocation::new(
                    tree_name, tree_index, tree_depth, node_name, node_index,
                ),
                dynamic_ref_name: dynamic_ref_name.to_string(),
                metadata_raw_str: dynamic_raw_str.to_string(),
                metadata_type: behavior_util::type_name::<T>().to_string(),
            }),
            Some(parsed) => Ok(DynamicCell::new(DummyMutable::new(parsed))),
        },
        Some(captures) => match captures.get(1) {
            None => Err(BehaviorError::RegexCapturesFailed {
                location: NodeLocation::new(
                    tree_name, tree_index, tree_depth, node_name, node_index,
                ),
                dynamic_ref_name: dynamic_ref_name.to_string(),
                pattern_str: "^<(.*?)>$".to_string(),
                capture_str: dynamic_raw_str.to_string(),
            }),
            Some(blackboard_name) => {
                let blackboard_name = blackboard_name.as_str();
                match blackboard_map.get(blackboard_name) {
                    None => Err(BehaviorError::DynamicBlackBoardNotFound {
                        location: NodeLocation::new(
                            tree_name, tree_index, tree_depth, node_name, node_index,
                        ),
                        dynamic_ref_name: dynamic_ref_name.to_string(),
                        blackboard_name: blackboard_name.to_string(),
                        blackboard_type: behavior_util::type_name::<T>().to_string(),
                    }),
                    Some(blackboard_value) => blackboard_value
                        .downcast_to_blackboard_cell::<T>()
                        .map_or_else(
                            || {
                                Err(BehaviorError::DynamicBlackBoardDowncastFailed {
                                    location: NodeLocation::new(
                                        tree_name, tree_index, tree_depth, node_name, node_index,
                                    ),
                                    dynamic_ref_name: dynamic_ref_name.to_string(),
                                    blackboard_name: blackboard_name.to_string(),
                                    blackboard_type: behavior_util::type_name::<T>().to_string(),
                                })
                            },
                            |cell| Ok(DynamicCell::new(cell)),
                        ),
                }
            }
        },
    }
}

pub trait BtNode: TreeNodeStatus + Debug {
    type Context;
    type World;
    type Entity;

    fn begin(
        &mut self,
        _ctx: &mut Self::Context,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) -> Status {
        Status::Success
    }

    fn end(
        &mut self,
        _ctx: &mut Self::Context,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) -> Status {
        Status::Success
    }

    fn tick(
        &mut self,
        _ctx: &mut Self::Context,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) -> Status;

    fn execute(
        &mut self,
        ctx: &mut Self::Context,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        if self.is_completed() {
            self.begin(ctx, world, entity);
        }

        let status = self.tick(ctx, world, entity);
        self.set_status(status);

        if self.is_completed() {
            self.end(ctx, world, entity);
        }
        status
    }

    #[inline]
    fn node_name(&self) -> &'static str {
        behavior_util::simplified_name::<Self>()
    }

    // fn clone_box(&self) -> Box<dyn BTNode>;
}

pub type Executor<C, W, E> = Box<dyn BtNode<Context = C, World = W, Entity = E>>;
#[derive(Debug)]
pub struct BtAction<C, W, E> {
    pub node_name: String,
    pub node_index: i32,
    pub executor: Executor<C, W, E>,
}
impl<C, W, E> BtAction<C, W, E> {
    pub fn from_executor(node_name: String, node_index: i32, executor: Executor<C, W, E>) -> Self {
        Self {
            node_name,
            executor,
            node_index,
        }
    }
}

impl<C, W, E> TreeNodeStatus for BtAction<C, W, E> {
    fn get_status(&self) -> Status {
        self.executor.get_status()
    }

    fn set_status(&mut self, status: Status) {
        self.executor.set_status(status);
    }

    fn reset_status(&mut self) {
        self.executor.reset_status();
    }

    fn is_running(&self) -> bool {
        self.executor.is_running()
    }

    fn is_completed(&self) -> bool {
        self.executor.is_completed()
    }
}

impl<C, W, E> TreeNode for BtAction<C, W, E> {
    type Action = BtAction<C, W, E>;
    type BlackBoardContext = C;
    type ActionTickFunc = ActionTickFunc<C, W, E>;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        _blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        _func: &mut Self::ActionTickFunc,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) -> Status {
        unreachable!("BTTask should never be used as control node")
    }

    fn action_tick(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        self.executor.execute(ctx, world, entity)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        self.executor.end(ctx, world, entity);
        self.reset_status();
    }

    fn node_index(&self) -> i32 {
        self.node_index
    }

    fn node_name(&self) -> &'static str {
        self.executor.node_name()
    }

    fn node_type(&self) -> NodeType {
        NodeType::LeafNode
    }

    fn children(
        &self,
    ) -> Vec<
        &Box<
            dyn TreeNode<
                Action = Self::Action,
                BlackBoardContext = Self::BlackBoardContext,
                ActionTickFunc = Self::ActionTickFunc,
                World = Self::World,
                Entity = Self::Entity,
            >,
        >,
    > {
        Vec::new()
    }
}

pub trait BtNodeGenerator {
    type Context;
    type World;
    type Entity;
    fn generate_node(
        tree_name: &str,
        tree_index: i32,
        tree_depth: i32,
        node_name: &str,
        node_idx: i32,
        blackboard_map: &BlackBoardMap,
        metadata_map: Option<&HashMap<String, String>>,
        blackbaord_ref_map: Option<&HashMap<String, String>>,
        dynamic_ref_map: Option<&HashMap<String, String>>,
    ) -> Result<Executor<Self::Context, Self::World, Self::Entity>, BehaviorError>;
}
