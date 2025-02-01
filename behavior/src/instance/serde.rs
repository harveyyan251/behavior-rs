use super::factory::{BtFactory, BtInstance, FastHashMap, ParentTreeLink, TreeState};
use super::node::BtAction;
use crate::template::{
    ActionNode, BranchCondNode, BranchNode, ForceFailureNode, ForceSuccessNode, IfNode,
    IfThenElseNode, InvertNode, LimiterNode, ParallelOrNode, ParallelSelectNode,
    ParallelSequenceNode, PriorityBranchNode, RepeatNode, RetryNode, SelectNode, SequenceNode,
    SubTreeNode, TimeoutNode, UntilFailureNode, UntilSuccessNode, WeightSelectNode, WhileNode,
};
cfg_expression_node!(
    use crate::template::ExpressionNode;
);
use crate::{
    AlwaysFailureNode, AlwaysSuccessNode, Behavior, BehaviorError, BlackBoard, BlackBoardMap,
    ImmediateRepeatNode, ImmediateRetryNode, ParallelAndNode, WaitForeverNode, WaitNode,
};
use ahash::HashMapExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlackBoardTemplate {
    pub bb_name: String,
    pub bb_type: String,
    pub bb_value: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ActionTemplate {
    pub name: String,
    pub meta_map: Option<HashMap<String, String>>,
    pub bb_ref_map: Option<HashMap<String, String>>,
    pub dyn_ref_map: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TreeTemplate {
    tree_blackboard: Option<Vec<BlackBoardTemplate>>,
    tree_structure: Behavior<ActionTemplate>,
}

impl TreeTemplate {
    fn to_tree_action<C: Unpin + Default + 'static, W: 'static, E: 'static>(
        factory: &BtFactory<C, W, E>,
        tree_name: &str,
        tree_index: i32,
        tree_depth: i32,
        node_index: &i32,
        action_template: &ActionTemplate,
        blackboard_map: &BlackBoardMap,
    ) -> Result<BtAction<C, W, E>, BehaviorError> {
        let blackboard_ref_map = action_template.bb_ref_map.as_ref();
        let dynamic_ref_map = action_template.dyn_ref_map.as_ref();
        let metadata_map = action_template.meta_map.as_ref();
        let executor = factory.get_node_executor(
            tree_name,
            tree_index,
            tree_depth,
            &action_template.name,
            *node_index,
            blackboard_map,
            metadata_map,
            blackboard_ref_map,
            dynamic_ref_map,
        )?;
        Ok(BtAction::from_executor(
            action_template.name.clone(),
            *node_index,
            executor,
        ))
    }

    fn to_tree_state<C: Unpin + Default + 'static, W: 'static, E: 'static>(
        factory: &BtFactory<C, W, E>,
        tree_name: &str,
        tree_index: i32,
        tree_depth: i32,
        behavior: &Behavior<ActionTemplate>,
        blackboard_map: &BlackBoardMap,
    ) -> Result<TreeState<C, W, E>, BehaviorError> {
        match behavior {
            Behavior::Action(node_index, action_template) => {
                let action_node = ActionNode::new(
                    *node_index,
                    Self::to_tree_action::<C, W, E>(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        node_index,
                        action_template,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(action_node))
            }
            Behavior::Wait(node_index, wait_time) => {
                let wait_node = WaitNode::new(*node_index, *wait_time);
                Ok(Box::new(wait_node))
            }
            Behavior::WaitForever(node_index) => {
                let wait_forever_node = WaitForeverNode::new(*node_index);
                Ok(Box::new(wait_forever_node))
            }
            Behavior::AlwaysSuccess(node_index) => {
                let always_success_node = AlwaysSuccessNode::new(*node_index);
                Ok(Box::new(always_success_node))
            }
            Behavior::AlwaysFailure(node_index) => {
                let always_failure_node = AlwaysFailureNode::new(*node_index);
                Ok(Box::new(always_failure_node))
            }
            #[cfg(feature = "expression_node")]
            Behavior::Expression(node_index, expression_str) => {
                // TODO: 使用 Result
                let expression_node =
                    ExpressionNode::new(*node_index, expression_str, blackboard_map);
                Ok(Box::new(expression_node))
            }
            Behavior::If(node_index, can_abort, cond, success) => {
                let if_node = IfNode::new(
                    *node_index,
                    *can_abort,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        cond,
                        blackboard_map,
                    )?,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        success,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(if_node))
            }
            Behavior::IfThenElse(node_index, can_abort, cond, success, failure) => {
                let if_node = IfThenElseNode::new(
                    *node_index,
                    *can_abort,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        cond,
                        blackboard_map,
                    )?,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        success,
                        blackboard_map,
                    )?,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        failure,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(if_node))
            }
            Behavior::While(node_index, while_cond, children) => {
                let while_node = WhileNode::new(
                    *node_index,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        while_cond,
                        blackboard_map,
                    )?,
                    children
                        .into_iter()
                        .map(|child| {
                            Self::to_tree_state(
                                factory,
                                tree_name,
                                tree_index,
                                tree_depth,
                                child,
                                blackboard_map,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(Box::new(while_node))
            }
            Behavior::Select(node_index, children) => {
                let select_node = SelectNode::new(
                    *node_index,
                    children
                        .into_iter()
                        .map(|child| {
                            Self::to_tree_state(
                                factory,
                                tree_name,
                                tree_index,
                                tree_depth,
                                child,
                                blackboard_map,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(Box::new(select_node))
            }
            Behavior::Sequence(node_index, children) => {
                let sequence_node = SequenceNode::new(
                    *node_index,
                    children
                        .into_iter()
                        .map(|child| {
                            Self::to_tree_state(
                                factory,
                                tree_name,
                                tree_index,
                                tree_depth,
                                child,
                                blackboard_map,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(Box::new(sequence_node))
            }
            Behavior::Branch(node_index, can_abort, branch_cond, branch_children) => {
                let branch_node = BranchNode::new(
                    *node_index,
                    *can_abort,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        branch_cond,
                        blackboard_map,
                    )?,
                    branch_children
                        .into_iter()
                        .map(|child| {
                            Self::to_tree_state(
                                factory,
                                tree_name,
                                tree_index,
                                tree_depth,
                                child,
                                blackboard_map,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(Box::new(branch_node))
            }
            Behavior::PriorityBranch(
                node_index,
                can_abort,
                branch_priorities,
                branch_cond,
                branch_children,
            ) => {
                let branch_node = PriorityBranchNode::new(
                    *node_index,
                    *can_abort,
                    branch_priorities,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        branch_cond,
                        blackboard_map,
                    )?,
                    branch_children
                        .into_iter()
                        .map(|child| {
                            Self::to_tree_state(
                                factory,
                                tree_name,
                                tree_index,
                                tree_depth,
                                child,
                                blackboard_map,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(Box::new(branch_node))
            }
            Behavior::BranchCond(node_index, branch_conds) => {
                let branch_node = BranchCondNode::new(
                    *node_index,
                    branch_conds
                        .into_iter()
                        .map(|child| {
                            Self::to_tree_state(
                                factory,
                                tree_name,
                                tree_index,
                                tree_depth,
                                child,
                                blackboard_map,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(Box::new(branch_node))
            }
            Behavior::ParallelAnd(node_index, children) => {
                let parallel_and_node = ParallelAndNode::new(
                    *node_index,
                    children
                        .into_iter()
                        .map(|child| {
                            Self::to_tree_state(
                                factory,
                                tree_name,
                                tree_index,
                                tree_depth,
                                child,
                                blackboard_map,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(Box::new(parallel_and_node))
            }
            Behavior::ParallelOr(node_index, children) => {
                let parallel_or_node = ParallelOrNode::new(
                    *node_index,
                    children
                        .into_iter()
                        .map(|child| {
                            Self::to_tree_state(
                                factory,
                                tree_name,
                                tree_index,
                                tree_depth,
                                child,
                                blackboard_map,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(Box::new(parallel_or_node))
            }
            Behavior::ParallelSequence(node_index, children) => {
                let parallel_sequence_node = ParallelSequenceNode::new(
                    *node_index,
                    children
                        .into_iter()
                        .map(|child| {
                            Self::to_tree_state(
                                factory,
                                tree_name,
                                tree_index,
                                tree_depth,
                                child,
                                blackboard_map,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(Box::new(parallel_sequence_node))
            }
            Behavior::ParallelSelect(node_index, children) => {
                let parallel_select_node = ParallelSelectNode::new(
                    *node_index,
                    children
                        .into_iter()
                        .map(|child| {
                            Self::to_tree_state(
                                factory,
                                tree_name,
                                tree_index,
                                tree_depth,
                                child,
                                blackboard_map,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(Box::new(parallel_select_node))
            }
            Behavior::WeightSelect(node_index, select_weights, select_children) => {
                let branch_node = WeightSelectNode::new(
                    *node_index,
                    select_weights,
                    select_children
                        .into_iter()
                        .map(|child| {
                            Self::to_tree_state(
                                factory,
                                tree_name,
                                tree_index,
                                tree_depth,
                                child,
                                blackboard_map,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(Box::new(branch_node))
            }
            Behavior::Invert(node_index, child) => {
                let invert_node = InvertNode::new(
                    *node_index,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        child,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(invert_node))
            }
            Behavior::ForceSuccess(node_index, child) => {
                let force_success_node = ForceSuccessNode::new(
                    *node_index,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        child,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(force_success_node))
            }
            Behavior::ForceFailure(node_index, child) => {
                let force_failure_node = ForceFailureNode::new(
                    *node_index,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        child,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(force_failure_node))
            }
            Behavior::UntilSuccess(node_index, child) => {
                let until_success_node = UntilSuccessNode::new(
                    *node_index,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        child,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(until_success_node))
            }
            Behavior::UntilFailure(node_index, child) => {
                let until_failure_node = UntilFailureNode::new(
                    *node_index,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        child,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(until_failure_node))
            }
            Behavior::TimeOut(node_index, timeout, child) => {
                let timeout_node = TimeoutNode::new(
                    *node_index,
                    *timeout,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        child,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(timeout_node))
            }
            Behavior::Limiter(node_index, duration, limit, child) => {
                let timeout_node = LimiterNode::new(
                    *node_index,
                    *duration,
                    *limit,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        child,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(timeout_node))
            }
            Behavior::Repeat(node_index, repeat_limit, child) => {
                let timeout_node = RepeatNode::new(
                    *node_index,
                    *repeat_limit,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        child,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(timeout_node))
            }
            Behavior::ImmediateRepeat(node_index, repeat_limit, child) => {
                let timeout_node = ImmediateRepeatNode::new(
                    *node_index,
                    *repeat_limit,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        child,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(timeout_node))
            }
            Behavior::Retry(node_index, attempts, child) => {
                let retry_node = RetryNode::new(
                    *node_index,
                    *attempts,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        child,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(retry_node))
            }
            Behavior::ImmediateRetry(node_index, attempts, child) => {
                let retry_node = ImmediateRetryNode::new(
                    *node_index,
                    *attempts,
                    Self::to_tree_state(
                        factory,
                        tree_name,
                        tree_index,
                        tree_depth,
                        child,
                        blackboard_map,
                    )?,
                );
                Ok(Box::new(retry_node))
            }
            Behavior::SubTree(node_index, subtree_name, parent_ref_map) => {
                let BtInstance {
                    tree_name,
                    tree_state,
                    tree_blackboard,
                    ..
                } = match factory.inner_create_tree_instance(
                    subtree_name,
                    *node_index,
                    tree_depth + 1,
                    Some(ParentTreeLink::new(
                        tree_name,
                        blackboard_map,
                        parent_ref_map,
                    )),
                ) {
                    Err(error) => {
                        return Err(BehaviorError::CreateSubTreeFailed {
                            node_index: *node_index,
                            parent_name: tree_name.to_string(),
                            subtree_name: subtree_name.to_string(),
                            create_error: Box::new(error),
                        })
                    }
                    Ok(instance) => instance,
                };
                let subtree_node =
                    SubTreeNode::new(*node_index, tree_name, tree_blackboard, tree_state);
                Ok(Box::new(subtree_node))
            }
        }
    }

    pub fn to_instance<C: Unpin + Default + 'static, W: 'static, E: 'static>(
        &self,
        factory: &BtFactory<C, W, E>,
        tree_name: &str,
        tree_index: i32,
        tree_depth: i32,
        parent_link_info: Option<(BlackBoardMap, &str)>,
    ) -> Result<(BlackBoard<C>, TreeState<C, W, E>), BehaviorError> {
        let tree_blackboard =
            self.to_tree_blackboard(factory, tree_name, tree_index, tree_depth, parent_link_info)?;
        let tree_state = Self::to_tree_state::<C, W, E>(
            factory,
            tree_name,
            tree_index,
            tree_depth,
            &self.tree_structure,
            tree_blackboard.blackboard_map_ref(),
        )?;
        Ok((tree_blackboard, tree_state))
    }

    pub fn to_tree_blackboard<C: Unpin + Default + 'static, W: 'static, E: 'static>(
        &self,
        factory: &BtFactory<C, W, E>,
        tree_name: &str,
        tree_index: i32,
        tree_depth: i32,
        parent_link: Option<(BlackBoardMap, &str)>,
    ) -> Result<BlackBoard<C>, BehaviorError> {
        let (mut blackboard_map, parent_tree_name) =
            parent_link.unwrap_or_else(|| (FastHashMap::new(), &""));
        if let Some(blackboard_template_list) = &self.tree_blackboard {
            for blackboard_template in blackboard_template_list {
                factory.init_blackboard(
                    tree_name,
                    tree_index,
                    tree_depth,
                    parent_tree_name,
                    &mut blackboard_map,
                    blackboard_template,
                )?;
            }
        }
        Ok(BlackBoard::new(
            C::default(),
            tree_name.to_string(),
            tree_index,
            tree_depth,
            blackboard_map,
        ))
    }
}
