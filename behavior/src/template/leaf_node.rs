#[cfg(feature = "expression_node")]
use super::blackboard::{BlackBoardMap, SharedBlackBoardValue};
use super::Status::{self, Failure, Running, Success};
use super::{BlackBoard, TreeNode, TreeNodeBase};
use super::{NodeType, TreeNodeStatus};
use behavior_macros::TreeNodeStatus;
#[cfg(feature = "expression_node")]
use evalexpr::*;
use std::marker::PhantomData;

#[derive(TreeNodeStatus)]
pub struct ActionNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    action: A,
    _marker: PhantomData<(C, W, E, F)>,
}
impl<A, C, F: ?Sized, W, E> ActionNode<A, C, F, W, E> {
    pub fn new(index: i32, action: A) -> Self {
        let base = TreeNodeBase::default();
        Self {
            base,
            index,
            action,
            _marker: PhantomData,
        }
    }
}
impl<A, C, F, W, E> TreeNode for ActionNode<A, C, F, W, E>
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
        let status = func(&mut self.action, blackboard, world, entity);
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
            self.action.reset(ctx, world, entity);
        }
    }

    fn node_index(&self) -> i32 {
        self.index
    }

    fn node_name(&self) -> &'static str {
        self.action.node_name()
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
        Vec::default()
    }

    cfg_tree_visualization! {
        fn visualize(
            &self,
            s: &mut String,
            // f: &mut std::fmt::Formatter<'_>,
            runtime: &super::Visualization,
            _is_last: bool,
            depth: usize,
        ) -> std::fmt::Result {
            self.action.visualize(s, runtime, true, depth)
        }
    }
}

#[derive(TreeNodeStatus)]
pub struct WaitNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    duration: i64,
    start_time: Option<i64>,
    _marker: PhantomData<(A, C, W, E, F)>,
}
impl<A, C, F: ?Sized, W, E> WaitNode<A, C, F, W, E> {
    pub fn new(index: i32, duration: i64) -> Self {
        let base = TreeNodeBase::default();
        let start_time = None;
        Self {
            base,
            index,
            duration,
            start_time,
            _marker: PhantomData,
        }
    }
}
impl<A, C, F, W, E> TreeNode for WaitNode<A, C, F, W, E>
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
        _func: &mut Self::ActionTickFunc,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) -> Status {
        let now = chrono::Utc::now().timestamp_millis();
        if let Some(start_time) = self.start_time {
            if now >= start_time + self.duration {
                self.start_time = None;
                return set_status!(self, blackboard, Success);
            }
        }
        if !self.is_running() {
            self.start_time = Some(now);
        }
        set_status!(self, blackboard, Running)
    }

    fn reset(
        &mut self,
        _ctx: &mut Self::BlackBoardContext,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
            self.start_time = None;
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
        Vec::default()
    }
}

#[derive(TreeNodeStatus)]
pub struct WaitForeverNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    _marker: PhantomData<(A, C, W, E, F)>,
}
impl<A, C, F: ?Sized, W, E> WaitForeverNode<A, C, F, W, E> {
    pub fn new(index: i32) -> Self {
        let base = TreeNodeBase::default();
        Self {
            base,
            index,
            _marker: PhantomData,
        }
    }
}
impl<A, C, F, W, E> TreeNode for WaitForeverNode<A, C, F, W, E>
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
        _func: &mut Self::ActionTickFunc,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) -> Status {
        set_status!(self, blackboard, Running)
    }

    fn reset(
        &mut self,
        _ctx: &mut Self::BlackBoardContext,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) {
        if self.is_running() {
            self.reset_status();
        }
    }

    fn node_index(&self) -> i32 {
        self.index
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
        Vec::default()
    }
}

#[derive(TreeNodeStatus)]
pub struct AlwaysSuccessNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    _marker: PhantomData<(A, C, W, E, F)>,
}
impl<A, C, F: ?Sized, W, E> AlwaysSuccessNode<A, C, F, W, E> {
    pub fn new(index: i32) -> Self {
        let base = TreeNodeBase::default();
        Self {
            base,
            index,
            _marker: PhantomData,
        }
    }
}
impl<A, C, F, W, E> TreeNode for AlwaysSuccessNode<A, C, F, W, E>
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
        #[allow(unused)] blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        _func: &mut Self::ActionTickFunc,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) -> Status {
        set_status!(self, blackboard, Success)
    }

    fn reset(
        &mut self,
        _ctx: &mut Self::BlackBoardContext,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) {
    }

    fn node_index(&self) -> i32 {
        self.index
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
        Vec::default()
    }
}

#[derive(TreeNodeStatus)]
pub struct AlwaysFailureNode<A, C, F: ?Sized, W, E> {
    base: TreeNodeBase,
    index: i32,
    _marker: PhantomData<(A, C, W, E, F)>,
}
impl<A, C, F: ?Sized, W, E> AlwaysFailureNode<A, C, F, W, E> {
    pub fn new(index: i32) -> Self {
        let base = TreeNodeBase::default();
        Self {
            base,
            index,
            _marker: PhantomData,
        }
    }
}
impl<A, C, F, W, E> TreeNode for AlwaysFailureNode<A, C, F, W, E>
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
        _func: &mut Self::ActionTickFunc,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) -> Status {
        set_status!(self, blackboard, Failure)
    }

    fn reset(
        &mut self,
        _ctx: &mut Self::BlackBoardContext,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) {
    }

    fn node_index(&self) -> i32 {
        self.index
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
        Vec::default()
    }
}

cfg_expression_node! {
    // TODO: 重构表达式节点
    struct ExpressionWrapper {
        expr: Node,
        variable_map: std::collections::HashMap<String, SharedBlackBoardValue>,
    }

    impl ExpressionWrapper {
        pub fn new(expr: &str, bb_map: &BlackBoardMap) -> Option<Self> {
            let expr = match build_operator_tree::<DefaultNumericTypes>(expr) {
                Ok(expr) => expr,
                Err(err) => {
                    tracing::error!(
                        "ExpressionWrapper::build_operator_tree failed, tracing::error={}",
                        err
                    );
                    return None;
                }
            };
            let mut variable_map = std::collections::HashMap::new();
            for var in expr.iter_variable_identifiers() {
                if !variable_map.contains_key(var) {
                    match bb_map.get(var) {
                        Some(value) => {
                            if value.is_expr_var() {
                                variable_map.insert(var.to_string(), value.clone());
                            } else {
                                tracing::error!(
                                    "ExpressionWrapper::new failed, not valid expr variable, var={}, type={}",
                                    var, value.bb_type()
                                );
                                return None;
                            }
                        }
                        None => {
                            tracing::error!(
                                "ExpressionWrapper::new failed, var not exist in blackboard, var={}",
                                var
                            );
                            return None;
                        }
                    }
                }
            }
            Some(Self { expr, variable_map })
        }

        pub fn eval(&mut self) -> Status {
            let mut context = HashMapContext::<DefaultNumericTypes>::new();
            for (var, value) in self.variable_map.iter() {
                let value = match value.get_as_f64() {
                    Some(val) => val,
                    None => {
                        tracing::error!("ExpressionWrapper::eval::get_as_f64 failed, var={}", var);
                        return Status::Failure;
                    }
                };
                let _ = context.set_value(var.clone(), Value::from_float(value));
            }
            let status = match self.expr.eval_with_context_mut(&mut context) {
                Err(err) => {
                    tracing::error!(
                        "ExpressionWrapper::eval_with_context_mut failed, tracing::error_info={}",
                        err
                    );
                    return Status::Failure;
                }
                Ok(res) => match res {
                    Value::Boolean(val) => Status::from(val),
                    _ => Status::Success,
                },
            };
            for (var, value) in self.variable_map.iter_mut() {
                match context.get_value(var) {
                    Some(val) => match val {
                        Value::Float(val) => {
                            if !value.set_from_f64(*val) {
                                tracing::error!("ExpressionWrapper::eval::set_from_f64 failed, type_name={}, var={}", value.bb_type(), var);
                                return Status::Failure;
                            };
                        }
                        _ => {
                            tracing::error!("ExpressionWrapper::eval failed, not Float var, var={}", var);
                            return Status::Failure;
                        }
                    },
                    None => {
                        tracing::error!("ExpressionWrapper::eval::get_value failed, var not exist in context, var={}", var);
                        return Status::Failure;
                    }
                }
            }
            status
        }
    }

    #[derive(TreeNodeStatus)]
    pub struct ExpressionNode<A, C, F: ?Sized, W, E> {
        base: TreeNodeBase,
        index: i32,
        expression_str: String,
        expression: Option<ExpressionWrapper>,
        _marker: PhantomData<(A, C, W, E, F)>,
    }
    impl<A, C, F: ?Sized, W, E> ExpressionNode<A, C, F, W, E> {
        pub fn new(idx: i32, expression_str: &str, bb_map: &BlackBoardMap) -> Self {
            let base = TreeNodeBase::default();
            let expression = ExpressionWrapper::new(expression_str, bb_map);
            let expression_str = expression_str.to_string();
            Self {
                base,
                index: idx,
                expression,
                expression_str,
                _marker: PhantomData,
            }
        }
    }
    impl<A, C, F, W, E> TreeNode for ExpressionNode<A, C, F, W, E>
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
            #[allow(unused)] blackboard: &mut BlackBoard<Self::BlackBoardContext>,
            _func: &mut Self::ActionTickFunc,
            _world: &mut Self::World,
            _entity: &Self::Entity,
        ) -> Status {
            let status = match &mut self.expression {
                Some(expr) => expr.eval(),
                None => {
                    tracing::error!(
                        "ExpressionNode::control_tick tracing::error, init failed, expression_str={}",
                        self.expression_str
                    );
                    Status::Failure
                }
            };
            set_status!(self, blackboard, status)
        }

        fn reset(
            &mut self,
            _ctx: &mut Self::BlackBoardContext,
            _world: &mut Self::World,
            _entity: &Self::Entity,
        ) {
        }

        fn node_index(&self) -> i32 {
            self.index
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
}
