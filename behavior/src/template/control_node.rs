use super::blackboard::BlackBoard;
use super::status::Status::*;
use super::tree_node::TreeNode;
use super::BranchData;
use super::NodeType;
use super::Status;
use super::TreeNodeBase;
use super::TreeNodeStatus;
use super::TreeNodeType;
use behavior_macros::TreeNodeStatus;
use behavior_util::weight_select_index;
use core::panic;
use rand::distr::weighted::WeightedIndex;
use std::collections::VecDeque;

static SUCCESS_BRANCH: usize = 0;
static FAILURE_BRANCH: usize = 1;

#[derive(TreeNodeStatus)]
pub struct IfNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    can_abort: bool,
    cond: TreeNodeType<A, C, F, W, E>,
    success: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> IfNode<A, C, F, W, E> {
    pub fn new(
        index: i32,
        can_abort: bool,
        cond: TreeNodeType<A, C, F, W, E>,
        success: TreeNodeType<A, C, F, W, E>,
    ) -> Self {
        let base = TreeNodeBase::default();
        Self {
            base,
            index,
            can_abort,
            cond,
            success,
        }
    }
}
impl<A, C, F, W, E> TreeNode for IfNode<A, C, F, W, E>
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
        let now_branch = if self.can_abort || !self.is_running() {
            match self.cond.control_tick(blackboard, func, world, entity) {
                Success => SUCCESS_BRANCH,
                Failure => FAILURE_BRANCH,
                Running => return set_status!(self, blackboard, Running),
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.cond.node_name(),
                    self.cond.node_index()
                ),
            }
        } else {
            SUCCESS_BRANCH
        };

        if self.is_running() && now_branch == FAILURE_BRANCH {
            self.success.reset(blackboard, world, entity);
        }

        let status = if now_branch == SUCCESS_BRANCH {
            self.success.control_tick(blackboard, func, world, entity)
        } else {
            Failure
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
            self.cond.reset(ctx, world, entity);
            self.success.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        vec![&self.cond, &self.success]
    }
}

#[derive(TreeNodeStatus)]
pub struct IfThenElseNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    can_abort: bool,
    // TODO: prev_branch 考虑改为 NonNull<TreeNodeType<A, C, F, W, E>>
    prev_branch: Option<usize>,
    cond: TreeNodeType<A, C, F, W, E>,
    success: TreeNodeType<A, C, F, W, E>,
    failure: TreeNodeType<A, C, F, W, E>,
}
impl<A, C, F: ?Sized, W, E> IfThenElseNode<A, C, F, W, E> {
    pub fn new(
        index: i32,
        can_abort: bool,
        cond: TreeNodeType<A, C, F, W, E>,
        success: TreeNodeType<A, C, F, W, E>,
        failure: TreeNodeType<A, C, F, W, E>,
    ) -> Self {
        let base = TreeNodeBase::default();
        let prev_branch = None;
        Self {
            base,
            index,
            can_abort,
            prev_branch,
            cond,
            success,
            failure,
        }
    }
}
impl<A, C, F, W, E> TreeNode for IfThenElseNode<A, C, F, W, E>
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
        let prev_branch = self.prev_branch.take();
        let now_branch = if self.can_abort || prev_branch.is_none() {
            match self.cond.control_tick(blackboard, func, world, entity) {
                Success => SUCCESS_BRANCH,
                Failure => FAILURE_BRANCH,
                Running => return set_status!(self, blackboard, Running),
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.cond.node_name(),
                    self.cond.node_index()
                ),
            }
        } else {
            prev_branch.unwrap()
        };

        if let Some(prev_branch) = prev_branch {
            if now_branch != prev_branch {
                if prev_branch == SUCCESS_BRANCH {
                    self.success.reset(blackboard, world, entity);
                } else {
                    self.failure.reset(blackboard, world, entity);
                }
            }
        }

        let status = if now_branch == SUCCESS_BRANCH {
            self.success.control_tick(blackboard, func, world, entity)
        } else {
            self.failure.control_tick(blackboard, func, world, entity)
        };

        if status == Running {
            self.prev_branch = Some(now_branch);
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
            self.cond.reset(ctx, world, entity);
            if let Some(prev_branch) = self.prev_branch.take() {
                if prev_branch == SUCCESS_BRANCH {
                    self.success.reset(ctx, world, entity);
                } else {
                    self.failure.reset(ctx, world, entity);
                }
            }
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        vec![&self.cond, &self.success, &self.failure]
    }
}

#[derive(TreeNodeStatus)]
pub struct BranchNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    can_abort: bool,
    prev_branch: Option<usize>,
    branch_cond: TreeNodeType<A, C, F, W, E>,
    branch_children: Vec<TreeNodeType<A, C, F, W, E>>,
}
impl<A, C, F: ?Sized, W, E> BranchNode<A, C, F, W, E> {
    pub fn new(
        index: i32,
        can_abort: bool,
        branch_cond: TreeNodeType<A, C, F, W, E>,
        branch_children: Vec<TreeNodeType<A, C, F, W, E>>,
    ) -> Self {
        assert!(
            branch_children.len() >= 2,
            "BranchNode' branch children must >= 2, node_index={}",
            index
        );
        let base = TreeNodeBase::default();
        let prev_branch = None;
        Self {
            base,
            index,
            can_abort,
            prev_branch,
            branch_cond,
            branch_children,
        }
    }
}
impl<A, C, F, W, E> TreeNode for BranchNode<A, C, F, W, E>
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
        let prev_branch = self.prev_branch.take();
        let now_branch = if self.can_abort || prev_branch.is_none() {
            match self
                .branch_cond
                .control_tick(blackboard, func, world, entity)
            {
                Success => SUCCESS_BRANCH,
                Failure => FAILURE_BRANCH,
                Running => return set_status!(self, blackboard, Running),
                Branch(data) => {
                    assert!(
                        data.is_single_branch(),
                        "BranchNode' branch condition child must return single branch"
                    );
                    data.get_single_branch()
                }
                _ => panic_if_idle!(
                    self.node_name(),
                    self.index,
                    self.branch_cond.node_name(),
                    self.branch_cond.node_index()
                ),
            }
        } else {
            prev_branch.unwrap()
        };

        let status = if now_branch < self.branch_children.len() {
            if let Some(prev_branch) = prev_branch {
                if now_branch != prev_branch {
                    self.branch_children[prev_branch].reset(blackboard, world, entity);
                }
            }
            self.branch_children[now_branch].control_tick(blackboard, func, world, entity)
        } else {
            eprintln!(
                "BranchNode::control_tick error, invalid branch, branch={}, children_len={}, node_name={}, node_index={}, child_name={}, child_index={}",
                now_branch,
                self.branch_children.len(),
                self.node_name(),
                self.node_index(),
                self.branch_cond.node_name(),
                self.branch_cond.node_index()
            );
            Failure
        };

        if status == Running {
            self.prev_branch = Some(now_branch);
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
            self.branch_cond.reset(ctx, world, entity);
            if let Some(prev_branch) = self.prev_branch.take() {
                self.branch_children[prev_branch].reset(ctx, world, entity);
            }
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        let mut children = vec![&self.branch_cond];
        self.branch_children
            .iter()
            .for_each(|child| children.push(child));
        children
    }
}
// TODO: MultipleBranchNode

#[derive(TreeNodeStatus)]
pub struct PriorityBranchNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    can_abort: bool,
    prev_branch: Option<usize>,
    branch_cond: TreeNodeType<A, C, F, W, E>,
    branch_priorities: Vec<i32>,
    branch_children: Vec<TreeNodeType<A, C, F, W, E>>,
}
impl<A, C, F: ?Sized, W, E> PriorityBranchNode<A, C, F, W, E> {
    pub fn new(
        index: i32,
        can_abort: bool,
        branch_priorities_str: &str,
        branch_cond: TreeNodeType<A, C, F, W, E>,
        branch_children: Vec<TreeNodeType<A, C, F, W, E>>,
    ) -> Self {
        assert!(
            branch_children.len() >= 2,
            "PriorityBranchNode' branch children must >= 2, node_index={}",
            index
        );
        let branch_priorities: Vec<_> = branch_priorities_str
            .split('|')
            .filter_map(|priority_str| priority_str.parse::<i32>().ok())
            .collect();
        assert!(
            branch_priorities.len() == branch_children.len(),
            "PriorityBranchNode' branch priorities must equal to branch children, node_index={}, branch_priorities_str={}, branch_children_len={}",
            index,
            branch_priorities_str,
            branch_children.len()
        );
        let base = TreeNodeBase::default();
        let prev_branch = None;
        Self {
            base,
            index,
            can_abort,
            prev_branch,
            branch_cond,
            branch_priorities,
            branch_children,
        }
    }
}
impl<A, C, F, W, E> TreeNode for PriorityBranchNode<A, C, F, W, E>
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
        let prev_branch = self.prev_branch.take();
        let mut now_branch = if self.can_abort || prev_branch.is_none() {
            match self
                .branch_cond
                .control_tick(blackboard, func, world, entity)
            {
                Success => SUCCESS_BRANCH,
                Failure => FAILURE_BRANCH,
                Running => return set_status!(self, blackboard, Running),
                Branch(data) => {
                    assert!(
                        data.is_single_branch(),
                        "PriorityBranchNode' branch condition child must return single branch"
                    );
                    data.get_single_branch()
                }
                _ => panic_if_idle!(
                    self.node_name(),
                    self.index,
                    self.branch_cond.node_name(),
                    self.branch_cond.node_index()
                ),
            }
        } else {
            prev_branch.unwrap()
        };

        let status = if now_branch < self.branch_children.len() {
            if let Some(prev_branch) = prev_branch {
                if self.branch_priorities[now_branch] <= self.branch_priorities[prev_branch] {
                    now_branch = prev_branch;
                }
                if now_branch != prev_branch {
                    self.branch_children[prev_branch].reset(blackboard, world, entity);
                }
            }
            self.branch_children[now_branch].control_tick(blackboard, func, world, entity)
        } else {
            eprintln!(
                "PriorityBranchNode::control_tick error, invalid branch, branch={}, children_len={}, node_name={}, node_index={}, child_name={}, child_index={}",
                now_branch,
                self.branch_children.len(),
                self.node_name(),
                self.index,
                self.branch_cond.node_name(),
                self.branch_cond.node_index()
            );
            Failure
        };

        if status == Running {
            self.prev_branch = Some(now_branch);
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
            self.branch_cond.reset(ctx, world, entity);
            if let Some(prev_branch) = self.prev_branch.take() {
                self.branch_children[prev_branch].reset(ctx, world, entity);
            }
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        let mut children = vec![&self.branch_cond];
        self.branch_children
            .iter()
            .for_each(|child| children.push(child));
        children
    }
}

#[derive(TreeNodeStatus)]
pub struct BranchCondNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    running_queue: Option<VecDeque<usize>>,
    branch_conditions: Vec<TreeNodeType<A, C, F, W, E>>,
}
impl<A, C, F: ?Sized, W, E> BranchCondNode<A, C, F, W, E> {
    pub fn new(index: i32, branch_conditions: Vec<TreeNodeType<A, C, F, W, E>>) -> Self {
        let base = TreeNodeBase::default();
        let running_queue = None;
        Self {
            base,
            index,
            running_queue,
            branch_conditions,
        }
    }
}
impl<A, C, F, W, E> TreeNode for BranchCondNode<A, C, F, W, E>
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
        let mut pending_queue = match self.running_queue.take() {
            Some(running_queue) => running_queue,
            None => (0..self.branch_conditions.len()).collect(),
        };
        let mut running_queue = VecDeque::new();
        while let Some(index) = pending_queue.pop_front() {
            match self.branch_conditions[index].control_tick(blackboard, func, world, entity) {
                Failure => {}
                Running => {
                    running_queue.push_back(index);
                }
                Success => {
                    for index in running_queue.into_iter() {
                        self.branch_conditions[index].reset(blackboard, world, entity);
                    }
                    if self.is_running() {
                        for index in pending_queue.into_iter() {
                            self.branch_conditions[index].reset(blackboard, world, entity);
                        }
                    }
                    let status = Branch(BranchData::single_branch(index));
                    return set_status!(self, blackboard, status);
                }
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.branch_conditions[index].node_name(),
                    self.branch_conditions[index].node_index()
                ),
            };
        }

        let status = if running_queue.is_empty() {
            Failure
        } else {
            self.running_queue = Some(running_queue);
            Running
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
            if let Some(running_queue) = self.running_queue.take() {
                for index in running_queue.into_iter() {
                    self.branch_conditions[index].reset(ctx, world, entity);
                }
            }
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        self.branch_conditions.iter().collect()
    }
}

#[derive(TreeNodeStatus)]
pub struct SelectNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    cursor: usize,
    children: Vec<TreeNodeType<A, C, F, W, E>>,
}
impl<A, C, F: ?Sized, W, E> SelectNode<A, C, F, W, E> {
    pub fn new(index: i32, children: Vec<TreeNodeType<A, C, F, W, E>>) -> Self {
        let base = TreeNodeBase::default();
        Self {
            base,
            index,
            cursor: 0,
            children,
        }
    }
}
impl<A, C, F, W, E> TreeNode for SelectNode<A, C, F, W, E>
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
        while self.cursor < self.children.len() {
            match self.children[self.cursor].control_tick(blackboard, func, world, entity) {
                Failure => self.cursor += 1,
                Running => return set_status!(self, blackboard, Running),
                Success => {
                    self.cursor = 0;
                    return set_status!(self, blackboard, Success);
                }
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.children[self.cursor].node_name(),
                    self.children[self.cursor].node_index()
                ),
            };
        }
        self.cursor = 0;
        set_status!(self, blackboard, Failure)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        self.reset_status();
        self.children[self.cursor].reset(ctx, world, entity);
        self.cursor = 0;
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        self.children.iter().collect()
    }
}

#[derive(TreeNodeStatus)]
pub struct SequenceNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    cursor: usize,
    children: Vec<TreeNodeType<A, C, F, W, E>>,
}
impl<A, C, F: ?Sized, W, E> SequenceNode<A, C, F, W, E> {
    pub fn new(index: i32, children: Vec<TreeNodeType<A, C, F, W, E>>) -> Self {
        let base = TreeNodeBase::default();
        Self {
            base,
            index,
            cursor: 0,
            children,
        }
    }
}
impl<A, C, F, W, E> TreeNode for SequenceNode<A, C, F, W, E>
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
        while self.cursor < self.children.len() {
            match self.children[self.cursor].control_tick(blackboard, func, world, entity) {
                Success => self.cursor += 1,
                Failure => {
                    self.cursor = 0;
                    return set_status!(self, blackboard, Failure);
                }
                Running => return set_status!(self, blackboard, Running),
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.children[self.cursor].node_name(),
                    self.children[self.cursor].node_index()
                ),
            };
        }
        self.cursor = 0;
        set_status!(self, blackboard, Success)
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.children[self.cursor].reset(ctx, world, entity);
            self.cursor = 0;
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        self.children.iter().collect()
    }
}

#[derive(TreeNodeStatus)]
pub struct WhileNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    while_cond: TreeNodeType<A, C, F, W, E>,
    children: Vec<TreeNodeType<A, C, F, W, E>>,
}
impl<A, C, F: ?Sized, W, E> WhileNode<A, C, F, W, E> {
    pub fn new(
        index: i32,
        while_cond: TreeNodeType<A, C, F, W, E>,
        children: Vec<TreeNodeType<A, C, F, W, E>>,
    ) -> Self {
        let base = TreeNodeBase::default();
        Self {
            base,
            index,
            while_cond,
            children,
        }
    }
}
impl<A, C, F, W, E> TreeNode for WhileNode<A, C, F, W, E>
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
        let status = match self
            .while_cond
            .control_tick(blackboard, func, world, entity)
        {
            Running => {
                self.children.iter_mut().for_each(|child| {
                    child.control_tick(blackboard, func, world, entity);
                });
                Running
            }
            status @ (Success | Failure) => {
                self.children.iter_mut().for_each(|child| {
                    child.reset(blackboard, world, entity);
                });
                status
            }
            _ => panic_if_idle_or_branch!(
                self.node_name(),
                self.index,
                self.while_cond.node_name(),
                self.while_cond.node_index()
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
            self.while_cond.reset(ctx, world, entity);
            self.children.iter_mut().for_each(|child| {
                child.reset(ctx, world, entity);
            });
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        let mut children = vec![&self.while_cond];
        self.children
            .iter()
            .for_each(|child| children.push(child));
        children
    }
}

#[derive(TreeNodeStatus)]
pub struct ParallelSequenceNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    result: Status,
    running_queue: Option<VecDeque<usize>>,
    children: Vec<TreeNodeType<A, C, F, W, E>>,
}
impl<A, C, F: ?Sized, W, E> ParallelSequenceNode<A, C, F, W, E> {
    pub fn new(index: i32, children: Vec<TreeNodeType<A, C, F, W, E>>) -> Self {
        let base = TreeNodeBase::default();
        let result = Success;
        let running_queue = None;
        Self {
            base,
            index,
            result,
            running_queue,
            children,
        }
    }
}
impl<A, C, F, W, E> TreeNode for ParallelSequenceNode<A, C, F, W, E>
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
        let mut pending_queue = match self.running_queue.take() {
            Some(running_queue) => running_queue,
            None => (0..self.children.len()).collect(),
        };

        let mut running_queue = VecDeque::new();
        while let Some(index) = pending_queue.pop_front() {
            match self.children[index].control_tick(blackboard, func, world, entity) {
                Success => {}
                Failure => self.result = Failure,
                Running => running_queue.push_back(index),
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.children[index].node_name(),
                    self.children[index].node_index()
                ),
            };
        }
        let status = if running_queue.is_empty() {
            self.result.replace(Success)
        } else {
            self.running_queue = Some(running_queue);
            Running
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
            self.result = Success;
            if let Some(running_queue) = self.running_queue.take() {
                for index in running_queue.into_iter() {
                    self.children[index].reset(ctx, world, entity);
                }
            }
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        self.children.iter().collect::<Vec<_>>()
    }
}

#[derive(TreeNodeStatus)]
pub struct ParallelSelectNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    result: Status,
    running_queue: Option<VecDeque<usize>>,
    children: Vec<TreeNodeType<A, C, F, W, E>>,
}
impl<A, C, F: ?Sized, W, E> ParallelSelectNode<A, C, F, W, E> {
    pub fn new(index: i32, children: Vec<TreeNodeType<A, C, F, W, E>>) -> Self {
        let base = TreeNodeBase::default();
        let result = Failure;
        let running_queue = None;
        Self {
            base,
            index,
            result,
            running_queue,
            children,
        }
    }
}
impl<A, C, F, W, E> TreeNode for ParallelSelectNode<A, C, F, W, E>
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
        let mut pending_queue = match self.running_queue.take() {
            Some(running_queue) => running_queue,
            None => (0..self.children.len()).collect(),
        };

        let mut running_queue = VecDeque::<usize>::new();
        while let Some(index) = pending_queue.pop_front() {
            match self.children[index].control_tick(blackboard, func, world, entity) {
                Failure => {}
                Success => self.result = Success,
                Running => running_queue.push_back(index),
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.children[index].node_name(),
                    self.children[index].node_index()
                ),
            };
        }
        let status = if running_queue.is_empty() {
            self.result.replace(Failure)
        } else {
            self.running_queue = Some(running_queue);
            Running
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
            self.result = Failure;
            if let Some(running_queue) = self.running_queue.take() {
                for index in running_queue.into_iter() {
                    self.children[index].reset(ctx, world, entity);
                }
            }
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        self.children.iter().collect::<Vec<_>>()
    }
}

#[derive(TreeNodeStatus)]
pub struct ParallelAndNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    running_queue: Option<VecDeque<usize>>,
    children: Vec<TreeNodeType<A, C, F, W, E>>,
}
impl<A, C, F: ?Sized, W, E> ParallelAndNode<A, C, F, W, E> {
    pub fn new(index: i32, children: Vec<TreeNodeType<A, C, F, W, E>>) -> Self {
        let base = TreeNodeBase::default();
        let running_queue = None;
        Self {
            base,
            index,
            running_queue,
            children,
        }
    }
}
impl<A, C, F, W, E> TreeNode for ParallelAndNode<A, C, F, W, E>
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
        let mut pending_queue = match self.running_queue.take() {
            Some(running_queue) => running_queue,
            None => (0..self.children.len()).collect(),
        };

        let mut running_queue = VecDeque::new();
        while let Some(index) = pending_queue.pop_front() {
            match self.children[index].control_tick(blackboard, func, world, entity) {
                Success => {}
                Running => running_queue.push_back(index),
                Failure => {
                    for index in running_queue.into_iter() {
                        self.children[index].reset(blackboard, world, entity);
                    }
                    if self.is_running() {
                        for index in pending_queue.into_iter() {
                            self.children[index].reset(blackboard, world, entity);
                        }
                    }
                    return set_status!(self, blackboard, Failure);
                }
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.children[index].node_name(),
                    self.children[index].node_index()
                ),
            };
        }
        let status = if running_queue.is_empty() {
            Success
        } else {
            self.running_queue = Some(running_queue);
            Running
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
            if let Some(running_queue) = self.running_queue.take() {
                for index in running_queue.into_iter() {
                    self.children[index].reset(ctx, world, entity);
                }
            }
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        self.children.iter().collect::<Vec<_>>()
    }
}

#[derive(TreeNodeStatus)]
pub struct ParallelOrNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    running_queue: Option<VecDeque<usize>>,
    children: Vec<TreeNodeType<A, C, F, W, E>>,
}
impl<A, C, F: ?Sized, W, E> ParallelOrNode<A, C, F, W, E> {
    pub fn new(index: i32, children: Vec<TreeNodeType<A, C, F, W, E>>) -> Self {
        let base = TreeNodeBase::default();
        let running_queue = None;
        Self {
            base,
            index,
            running_queue,
            children,
        }
    }
}
impl<A, C, F, W, E> TreeNode for ParallelOrNode<A, C, F, W, E>
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
        let mut pending_queue = match self.running_queue.take() {
            Some(running_queue) => running_queue,
            None => (0..self.children.len()).collect(),
        };

        let mut running_queue = VecDeque::new();
        while let Some(index) = pending_queue.pop_front() {
            match self.children[index].control_tick(blackboard, func, world, entity) {
                Failure => {}
                Running => running_queue.push_back(index),
                Success => {
                    for index in running_queue.into_iter() {
                        self.children[index].reset(blackboard, world, entity);
                    }
                    if self.is_running() {
                        for index in pending_queue.into_iter() {
                            self.children[index].reset(blackboard, world, entity);
                        }
                    }
                    return set_status!(self, blackboard, Success);
                }
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.children[index].node_name(),
                    self.children[index].node_index()
                ),
            };
        }
        let status = if running_queue.is_empty() {
            Failure
        } else {
            self.running_queue = Some(running_queue);
            Running
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
            if let Some(running_queue) = self.running_queue.take() {
                for index in running_queue.into_iter() {
                    self.children[index].reset(ctx, world, entity);
                }
            }
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        self.children.iter().collect::<Vec<_>>()
    }
}

#[derive(TreeNodeStatus)]
pub struct WeightSelectNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    prev_branch: Option<usize>,
    select_weights: Vec<f64>,
    select_children: Vec<TreeNodeType<A, C, F, W, E>>,
}
impl<A, C, F: ?Sized, W, E> WeightSelectNode<A, C, F, W, E> {
    pub fn new(
        index: i32,
        select_weights_str: &str,
        select_children: Vec<TreeNodeType<A, C, F, W, E>>,
    ) -> Self {
        assert!(
            !select_children.is_empty(),
            "WeightSelectNode' select children must not be empty, node_index={}",
            index
        );
        let select_weights: Vec<_> = select_weights_str
            .split('|')
            .filter_map(|weight_str| weight_str.parse::<f64>().ok())
            .collect();
        assert!(
            select_weights.len() == select_children.len(),
            "WeightSelectNode' branch priorities must equal to branch children, node_index={}, branch_priorities_str={}, branch_children_len={}",
            index,
            select_weights_str,
            select_children.len()
        );
        if let Err(error) = WeightedIndex::new(&select_weights) {
            panic!(
                "WeightSelectNode' select weights is not invalid, node_index={}, select_weights_str={}, error={:?}",
                index, 
                select_weights_str,
                error
            );
        }
        let base = TreeNodeBase::default();
        let prev_branch = None;
        Self {
            base,
            index,
            prev_branch,
            select_weights,
            select_children,
        }
    }
}
impl<A, C, F, W, E> TreeNode for WeightSelectNode<A, C, F, W, E>
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
        let now_branch = match self.prev_branch.take() {
            Some(prev_branch) => prev_branch,
            None => weight_select_index(&self.select_weights).unwrap(),
        };

        let status =
            match self.select_children[now_branch].control_tick(blackboard, func, world, entity) {
                status @ (Success | Failure | Running) => status,
                _ => panic_if_idle_or_branch!(
                    self.node_name(),
                    self.index,
                    self.select_children[now_branch].node_name(),
                    self.select_children[now_branch].node_index()
                ),
            };

        if status == Running {
            self.prev_branch = Some(now_branch);
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
            if let Some(running_branch) = self.prev_branch.take() {
                self.select_children[running_branch].reset(ctx, world, entity);
            }
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_type(&self) -> NodeType {
        NodeType::ControlNode
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
        self.select_children.iter().collect::<Vec<_>>()
    }
}
