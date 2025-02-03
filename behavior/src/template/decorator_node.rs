use super::status::Status::*;
use super::{
    BlackBoard, BlackBoardMap, SharedBlackBoardValue, Status, TreeNode, TreeNodeBase, TreeNodeType,
};
use super::{NodeType, TreeNodeStatus};
use behavior_macros::TreeNodeStatus;

#[derive(TreeNodeStatus)]
pub struct InvertNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> InvertNode<A, C, F, W, E> {
    pub fn new(index: i32, child: TreeNodeType<A, C, F, W, E>) -> Self {
        let base = TreeNodeBase::default();
        Self { base, index, child }
    }
}
impl<A, C, F, W, E> TreeNode for InvertNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let status = match self.child.control_tick(blackboard, func, world, entity) {
            Running => Running,
            Failure => Success,
            Success => Failure,
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.child.node_name(),
                self.child.node_index()
            ),
        };
        set_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct ForceSuccessNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> ForceSuccessNode<A, C, F, W, E> {
    pub fn new(index: i32, child: TreeNodeType<A, C, F, W, E>) -> Self {
        let base = TreeNodeBase::default();
        Self { base, index, child }
    }
}
impl<A, C, F, W, E> TreeNode for ForceSuccessNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let status = match self.child.control_tick(blackboard, func, world, entity) {
            Running => Running,
            Success | Failure => Success,
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.child.node_name(),
                self.child.node_index()
            ),
        };
        set_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct ForceFailureNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> ForceFailureNode<A, C, F, W, E> {
    pub fn new(index: i32, child: TreeNodeType<A, C, F, W, E>) -> Self {
        let base = TreeNodeBase::default();
        Self { base, index, child }
    }
}
impl<A, C, F, W, E> TreeNode for ForceFailureNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let status = match self.child.control_tick(blackboard, func, world, entity) {
            Running => Running,
            Success | Failure => Failure,
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.child.node_name(),
                self.child.node_index()
            ),
        };
        set_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct UntilSuccessNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> UntilSuccessNode<A, C, F, W, E> {
    pub fn new(index: i32, child: TreeNodeType<A, C, F, W, E>) -> Self {
        let base = TreeNodeBase::default();
        Self { base, index, child }
    }
}
impl<A, C, F, W, E> TreeNode for UntilSuccessNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let status = match self.child.control_tick(blackboard, func, world, entity) {
            Success => Success,
            Running | Failure => Running,
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.child.node_name(),
                self.child.node_index()
            ),
        };
        set_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct UntilFailureNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> UntilFailureNode<A, C, F, W, E> {
    pub fn new(index: i32, child: TreeNodeType<A, C, F, W, E>) -> Self {
        let base = TreeNodeBase::default();
        Self { base, index, child }
    }
}
impl<A, C, F, W, E> TreeNode for UntilFailureNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let status = match self.child.control_tick(blackboard, func, world, entity) {
            Failure => Failure,
            Running | Success => Running,
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.child.node_name(),
                self.child.node_index()
            ),
        };
        set_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct TimeoutNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    timeout: i64,
    start_time: Option<i64>,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> TimeoutNode<A, C, F, W, E> {
    pub fn new(index: i32, timeout: i64, child: TreeNodeType<A, C, F, W, E>) -> Self {
        assert!(
            timeout > 0,
            "TimeoutNode argument timeout must be greater than 0, index={}, timeout={}",
            index,
            timeout
        );
        let base = TreeNodeBase::default();
        let start_time = None;
        Self {
            base,
            index,
            timeout,
            start_time,
            child,
        }
    }
}
impl<A, C, F, W, E> TreeNode for TimeoutNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let now = chrono::Utc::now().timestamp_millis();
        if self.start_time.is_some() && now >= self.start_time.unwrap() + self.timeout {
            self.start_time = None;
            self.child.reset(blackboard, world, entity);
            return set_status!(self, blackboard, Failure);
        }

        let status = match self.child.control_tick(blackboard, func, world, entity) {
            status @ (Success | Failure) => {
                self.start_time = None;
                status
            }
            Running => {
                if !self.is_running() {
                    self.start_time = Some(now);
                }
                Running
            }
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.child.node_name(),
                self.child.node_index()
            ),
        };
        set_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.start_time = None;
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct LimiterNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    duration: i64,
    start_time: Option<i64>,
    max_executions: i32,
    execution_count: i32,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> LimiterNode<A, C, F, W, E> {
    pub fn new(
        index: i32,
        duration: i64,
        max_executions: i32,
        child: TreeNodeType<A, C, F, W, E>,
    ) -> Self {
        assert!(
            duration > 0 && max_executions > 0,
            "LimiterNode argument duration and max_executions must be greater than 0, index={}, duration={}, max_executions={}",
            index,
            duration,
            max_executions
        );
        let base = TreeNodeBase::default();
        let start_time = None;
        let execution_count = 0;
        Self {
            base,
            index,
            duration,
            start_time,
            max_executions,
            execution_count,
            child,
        }
    }
}
impl<A, C, F, W, E> TreeNode for LimiterNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let now = chrono::Utc::now().timestamp_millis();
        if self.start_time.is_some() && now >= (self.start_time.unwrap() + self.duration) {
            self.start_time = None;
            self.execution_count = 0;
        }
        if self.execution_count >= self.max_executions {
            return set_status!(self, blackboard, Failure);
        }

        let status = match self.child.control_tick(blackboard, func, world, entity) {
            Running => Running,
            status @ (Success | Failure) => {
                self.execution_count += 1;
                status
            }
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.child.node_name(),
                self.child.node_index()
            ),
        };
        if self.start_time.is_none() {
            self.start_time = Some(now);
        }
        set_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.start_time = None;
            self.execution_count = 0;
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct RepeatNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    max_repeats: i32,
    repeat_count: i32,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> RepeatNode<A, C, F, W, E> {
    pub fn new(index: i32, max_repeats: i32, child: TreeNodeType<A, C, F, W, E>) -> Self {
        assert!(
            max_repeats == -1 || max_repeats > 0,
            "RepeatNode argument max_repeats must be -1 or greater than 0, index={}, max_repeats={}",
            index,
            max_repeats
        );
        let base = TreeNodeBase::default();
        let repeat_count = 0;
        Self {
            base,
            index,
            repeat_count,
            max_repeats,
            child,
        }
    }
}
impl<A, C, F, W, E> TreeNode for RepeatNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let status = match self.child.control_tick(blackboard, func, world, entity) {
            Running => Running,
            status @ (Success | Failure) => {
                if self.max_repeats != -1 {
                    self.repeat_count += 1;
                }
                if self.repeat_count < self.max_repeats || self.max_repeats == -1 {
                    Running
                } else {
                    self.repeat_count = 0;
                    status
                }
            }
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.child.node_name(),
                self.child.node_index()
            ),
        };
        set_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.repeat_count = 0;
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct ImmediateRepeatNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    max_repeats: i32,
    repeat_count: i32,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> ImmediateRepeatNode<A, C, F, W, E> {
    pub fn new(index: i32, max_repeats: i32, child: TreeNodeType<A, C, F, W, E>) -> Self {
        assert!(
            max_repeats == -1 || max_repeats > 0,
            "ImmediateRepeatNode argument max_repeats must be -1 or greater than 0, index={}, max_repeats={}",
            index,
            max_repeats
        );
        let base = TreeNodeBase::default();
        let repeat_count = 0;
        Self {
            base,
            index,
            repeat_count,
            max_repeats,
            child,
        }
    }
}
impl<A, C, F, W, E> TreeNode for ImmediateRepeatNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let status = loop {
            match self.child.control_tick(blackboard, func, world, entity) {
                Running => break Running,
                status @ (Success | Failure) => {
                    if self.max_repeats != -1 {
                        self.repeat_count += 1;
                    }
                    if !(self.repeat_count < self.max_repeats || self.max_repeats == -1) {
                        self.repeat_count = 0;
                        break status;
                    }
                }
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.child.node_name(),
                    self.child.node_index()
                ),
            };
        };

        set_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.repeat_count = 0;
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct RetryNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    try_count: i32,
    max_attempts: i32,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> RetryNode<A, C, F, W, E> {
    pub fn new(index: i32, max_attempts: i32, child: TreeNodeType<A, C, F, W, E>) -> Self {
        assert!(
            max_attempts == -1 || max_attempts > 0,
            "RetryNode argument max_attempts must be -1 or greater than 0, index={}, max_attempts={}",
            index,
            max_attempts
        );
        let base = TreeNodeBase::default();
        let try_count = 0;
        Self {
            base,
            index,
            try_count,
            max_attempts,
            child,
        }
    }
}
impl<A, C, F, W, E> TreeNode for RetryNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let status = match self.child.control_tick(blackboard, func, world, entity) {
            Running => Running,
            Failure => {
                if self.max_attempts != -1 {
                    self.try_count += 1;
                }
                if self.try_count < self.max_attempts || self.max_attempts == -1 {
                    Running
                } else {
                    self.try_count = 0;
                    Failure
                }
            }
            Success => {
                self.try_count = 0;
                Success
            }
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.child.node_name(),
                self.child.node_index()
            ),
        };
        set_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.try_count = 0;
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct ImmediateRetryNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    try_count: i32,
    max_attempts: i32,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> ImmediateRetryNode<A, C, F, W, E> {
    pub fn new(index: i32, max_attempts: i32, child: TreeNodeType<A, C, F, W, E>) -> Self {
        assert!(
            max_attempts == -1 || max_attempts > 0,
            "ImmediateRetryNode argument max_attempts must be -1 or greater than 0, index={}, max_attempts={}",
            index,
            max_attempts
        );
        let base = TreeNodeBase::default();
        let try_count = 0;
        Self {
            base,
            index,
            try_count,
            max_attempts,
            child,
        }
    }
}
impl<A, C, F, W, E> TreeNode for ImmediateRetryNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let status = loop {
            match self.child.control_tick(blackboard, func, world, entity) {
                Running => break Running,
                Failure => {
                    if self.max_attempts != -1 {
                        self.try_count += 1;
                    }
                    if !(self.try_count < self.max_attempts || self.max_attempts == -1) {
                        self.try_count = 0;
                        break Failure;
                    }
                }
                Success => {
                    self.try_count = 0;
                    break Success;
                }
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.child.node_name(),
                    self.child.node_index()
                ),
            };
        };
        set_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.try_count = 0;
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct LogNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    blackboards: Vec<SharedBlackBoardValue>,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> LogNode<A, C, F, W, E> {
    pub fn new(
        index: i32,
        blackboards_str: &str,
        blackboard_map: &BlackBoardMap,
        child: TreeNodeType<A, C, F, W, E>,
    ) -> Self {
        let base = TreeNodeBase::default();
        let blackboards: Vec<_> = blackboards_str
            .split('|')
            .map(
                |blackbaord_name| match blackboard_map.get(blackbaord_name).cloned() {
                    Some(value) => value,
                    None => panic!(
                        "LogNode blackboard not found, blackboard_name={}, blackboards_str={}",
                        blackbaord_name, blackboards_str
                    ),
                },
            )
            .collect();
        Self {
            base,
            index,
            blackboards,
            child,
        }
    }
}
impl<A, C, F, W, E> TreeNode for LogNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        let status = match self.child.control_tick(blackboard, func, world, entity) {
            status @ (Success | Failure | Running) => status,
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.child.node_name(),
                self.child.node_index()
            ),
        };
        self.blackboards.iter().for_each(|value| {
            println!("\n{:?}", value);
        });
        set_subtree_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }
}

#[derive(TreeNodeStatus)]
pub struct SubTreeNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    #[allow(dead_code)]
    tree_name: String,
    blackboard: BlackBoard<C>,
    child: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> SubTreeNode<A, C, F, W, E> {
    pub fn new(
        index: i32,
        tree_name: String,
        blackboard: BlackBoard<C>,
        child: TreeNodeType<A, C, F, W, E>,
    ) -> Self {
        let base = TreeNodeBase::default();
        Self {
            base,
            index,
            tree_name,
            blackboard,
            child,
        }
    }
}
impl<A, C, F, W, E> TreeNode for SubTreeNode<A, C, F, W, E>
where
    A: TreeNode<BlackBoardContext = C, World = W, Entity = E>,
    F: ?Sized + FnMut(&mut A, &mut BlackBoard<C>, &mut W, &E) -> Status,
{
    type Action = A;
    type BlackBoardContext = C;
    type ActionTickFunc = F;
    type World = W;
    type Entity = E;

    fn control_tick(
        &mut self,
        #[allow(unused_variables)] blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status {
        #[cfg(feature = "tree_visualization")]
        self.blackboard.reset_visualization();
        let status = match self
            .child
            .control_tick(&mut self.blackboard, func, world, entity)
        {
            status @ (Success | Failure | Running) => status,
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.child.node_name(),
                self.child.node_index()
            ),
        };
        set_subtree_status!(self, blackboard, status)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            // self.blackboard.reset_runtime();
            self.child.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::DecoratorNode
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
        vec![&self.child]
    }

    cfg_tree_visualization! {
        fn visualize(
            &self,
            s: &mut String,
            // f: &mut std::fmt::Formatter<'_>,
            runtime: &super::Visualization,
            is_last: bool,
            depth: usize,
        ) -> std::fmt::Result {
            use std::fmt::Write;

            let prefix = if depth == 0 {
                "".to_string()
            } else {
                "│   ".repeat(depth - 1) + if is_last { "└── " } else { "├── " }
            };

            write!(
                s,
                "{}[{}]{}({}):{}\n",
                prefix,
                self.node_index(),
                self.node_name(),
                self.tree_name,
                runtime.get_node_status(self.node_index()).as_str()
            )?;
            let runtime = self.blackboard.visualization_ref();
            let children = self.children();
            let children_len = children.len();
            for (i, child) in children.into_iter().enumerate() {
                let is_last = i == children_len - 1;
                child.visualize(s, runtime, is_last, depth + 1)?;
            }
            Ok(())
        }
    }
}
