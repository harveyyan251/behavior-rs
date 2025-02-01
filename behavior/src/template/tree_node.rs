use super::blackboard::BlackBoard;
use super::status::Status::*;
use super::Status;
use std::fmt::Debug;
#[cfg(feature = "tree_visualization")]
use std::fmt::Write;

pub enum NodeType {
    ControlNode,
    DecoratorNode,
    LeafNode,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::ControlNode => "ControlNode",
            NodeType::DecoratorNode => "DecoratorNode",
            NodeType::LeafNode => "LeafNode",
        }
    }
}

pub trait TreeNodeStatus {
    fn get_status(&self) -> Status;
    fn set_status(&mut self, status: Status);
    fn reset_status(&mut self);
    fn is_running(&self) -> bool;
    fn is_completed(&self) -> bool;
}

#[derive(Clone, Debug, Default)]
pub struct TreeNodeBase {
    pub status: Status,
}

impl TreeNodeStatus for TreeNodeBase {
    fn get_status(&self) -> Status {
        self.status
    }
    fn set_status(&mut self, status: Status) {
        assert!(
            status != Idle,
            "Not allowed to manually set the status to Idle"
        );
        self.status = status;
    }
    fn reset_status(&mut self) {
        self.status = Idle;
    }
    fn is_running(&self) -> bool {
        self.status == Running
    }
    fn is_completed(&self) -> bool {
        self.status != Running && self.status != Idle
    }
}

pub trait TreeNode: TreeNodeStatus {
    type Action: TreeNode;
    type BlackBoardContext;
    type ActionTickFunc: ?Sized;
    type World;
    type Entity;

    // tick for control node
    fn control_tick(
        &mut self,
        blackboard: &mut BlackBoard<Self::BlackBoardContext>,
        func: &mut Self::ActionTickFunc,
        world: &mut Self::World,
        entity: &Self::Entity,
    ) -> Status;

    // tick for action node
    fn action_tick(
        &mut self,
        _ctx: &mut Self::BlackBoardContext,
        _world: &mut Self::World,
        _entity: &Self::Entity,
    ) -> Status {
        Success
    }

    fn reset(
        &mut self,
        ctx: &mut Self::BlackBoardContext,
        world: &mut Self::World,
        entity: &Self::Entity,
    );

    fn node_index(&self) -> i32;

    fn node_type(&self) -> NodeType;

    fn node_name(&self) -> &'static str {
        behavior_util::simplified_name::<Self>()
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
    >;

    cfg_tree_visualization! {
        fn visualize(
            &self,
            s: &mut String,
            // f: &mut std::fmt::Formatter<'_>,
            viz: &super::Visualization,
            is_last: bool,
            depth: usize,
        ) -> std::fmt::Result {
            let prefix = if depth == 0 {
                "".to_string()
            } else {
                "│   ".repeat(depth - 1) + if is_last { "└── " } else { "├── " }
            };

            write!(
                s,
                "{}[{}]{}:{}\n",
                prefix,
                self.node_index(),
                self.node_name(),
                viz.get_node_status(self.node_index()).as_str()
            )?;
            let children = self.children();
            let children_len = children.len();
            for (i, child) in children.into_iter().enumerate() {
                let is_last = i == children_len - 1;
                child.visualize(s, viz, is_last, depth + 1)?;
            }
            Ok(())
        }
    }
}

pub type TreeNodeType<A, C, F, W, E> =
    Box<dyn TreeNode<Action = A, BlackBoardContext = C, ActionTickFunc = F, World = W, Entity = E>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::FastHashMap;
    use ahash::HashMapExt;
    use std::marker::PhantomData;

    pub trait TestStateNode {
        type TickAction;
        type BlackBoardValue: Default;
        type TickFunc: ?Sized;

        fn tick(
            &mut self,
            blackboard: &mut BlackBoard<Self::BlackBoardValue>,
            func: &mut Self::TickFunc,
        ) -> (Status, f64);
    }

    #[derive(Debug)]
    pub struct TestActionNode<A, B, F: ?Sized> {
        _idx: i32,
        action: A,
        _marker: PhantomData<(B, F)>,
    }
    impl<A, B, F: ?Sized> TestActionNode<A, B, F> {
        pub fn new(_idx: i32, action: A) -> Self {
            Self {
                _idx,
                action,
                _marker: PhantomData,
            }
        }
    }
    impl<A, B, F> TestStateNode for TestActionNode<A, B, F>
    where
        B: Default,
        F: ?Sized + FnMut(&mut A, &mut BlackBoard<B>, f64) -> (Status, f64),
    {
        type TickAction = A;
        type BlackBoardValue = B;
        type TickFunc = F;

        fn tick(
            &mut self,
            blackboard: &mut BlackBoard<Self::BlackBoardValue>,
            func: &mut Self::TickFunc,
        ) -> (Status, f64) {
            println!("TestActionNode::tick");
            let (status, dt) = func(&mut self.action, blackboard, 0.0);
            // blackboard.update_node_status(self.idx, status);
            (status, dt)
        }
    }

    pub struct TestInvertNode<A, B, F: ?Sized> {
        _idx: i32,
        child: Box<dyn TestStateNode<TickAction = A, BlackBoardValue = B, TickFunc = F>>,
    }
    impl<A, B, F: ?Sized> TestInvertNode<A, B, F> {
        pub fn new(
            _idx: i32,
            child: Box<dyn TestStateNode<TickAction = A, BlackBoardValue = B, TickFunc = F>>,
        ) -> Self {
            Self { _idx, child }
        }
    }
    impl<A, B, F> TestStateNode for TestInvertNode<A, B, F>
    where
        B: Default,
        F: ?Sized + FnMut(&mut A, &mut BlackBoard<B>, f64) -> (Status, f64),
    {
        type TickAction = A;
        type BlackBoardValue = B;
        type TickFunc = F;

        fn tick(
            &mut self,
            blackboard: &mut BlackBoard<Self::BlackBoardValue>,
            func: &mut Self::TickFunc,
        ) -> (Status, f64) {
            println!("TestInvertNode::tick");
            let (status, dt) = match self.child.tick(blackboard, func) {
                (Running, dt) => (Running, dt),
                (Failure, dt) => (Success, dt),
                (Success, dt) => (Failure, dt),
                (Branch(_), dt) => {
                    println!("InvertNode::error, should not return branch status!");
                    (Failure, dt)
                }
                (Idle, dt) => {
                    println!("InvertNode::error, should not return Idle status!");
                    (Failure, dt)
                }
            };
            // blackboard.update_node_status(self.idx, status);
            (status, dt)
        }
    }

    #[test]
    fn dyn_trait_test() {
        type RealTickFunc = dyn FnMut(&mut i32, &mut BlackBoard<i32>, f64) -> (Status, f64);
        type Elem = Box<
            dyn TestStateNode<TickAction = i32, BlackBoardValue = i32, TickFunc = RealTickFunc>,
        >;
        let mut vec = Vec::<Elem>::new();
        let node1: Elem = Box::new(TestActionNode::<i32, i32, RealTickFunc>::new(1, 100));
        let node2: Elem = Box::new(TestActionNode::<i32, i32, RealTickFunc>::new(2, 200));
        let node3: Elem = Box::new(TestInvertNode::<i32, i32, RealTickFunc>::new(3, node2));
        vec.push(node1);
        vec.push(node3);

        let mut blackboard = BlackBoard::new(100, String::from("Test"), 0, 0, FastHashMap::new());
        // let mut closure = Box::new(
        //     |task: &mut i32, blackboard: &mut BlackBoard<i32>, dt: f64| {
        //         println!("task={}, blackboard={:?}", task, blackboard);
        //         RUNNING
        //     },
        // );
        // let mut closure = |task: &mut i32, blackboard: &mut BlackBoard<i32>, dt: f64| {
        //     println!("task={}, blackboard={:?}", task, blackboard);
        //     RUNNING
        // };
        vec.iter_mut().for_each(|node| {
            node.tick(
                &mut blackboard,
                &mut |task: &mut i32, blackboard: &mut BlackBoard<i32>, _: f64| {
                    println!("task={}, blackboard={:?}", task, blackboard);
                    (Running, 0.0)
                },
            );
        });
    }
}
