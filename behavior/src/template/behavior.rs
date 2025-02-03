use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum Behavior<A> {
    // Leaf Node
    Action(i32, A),
    Wait(i32, i64),
    WaitForever(i32),
    AlwaysSuccess(i32),
    AlwaysFailure(i32),
    #[cfg(feature = "expression_node")]
    Expression(i32, String),
    // Control Node
    If(i32, bool, Box<Behavior<A>>, Box<Behavior<A>>),
    IfThenElse(
        i32,
        bool,
        Box<Behavior<A>>,
        Box<Behavior<A>>,
        Box<Behavior<A>>,
    ),
    While(i32, Box<Behavior<A>>, Vec<Behavior<A>>),
    Select(i32, Vec<Behavior<A>>),
    Sequence(i32, Vec<Behavior<A>>),
    Branch(i32, bool, Box<Behavior<A>>, Vec<Behavior<A>>),
    PriorityBranch(i32, bool, String, Box<Behavior<A>>, Vec<Behavior<A>>),
    BranchCond(i32, Vec<Behavior<A>>),
    ParallelAnd(i32, Vec<Behavior<A>>),
    ParallelOr(i32, Vec<Behavior<A>>),
    ParallelSequence(i32, Vec<Behavior<A>>),
    ParallelSelect(i32, Vec<Behavior<A>>),
    WeightSelect(i32, String, Vec<Behavior<A>>),
    // Decorator Node
    Invert(i32, Box<Behavior<A>>),
    ForceFailure(i32, Box<Behavior<A>>),
    ForceSuccess(i32, Box<Behavior<A>>),
    UntilSuccess(i32, Box<Behavior<A>>),
    UntilFailure(i32, Box<Behavior<A>>),
    TimeOut(i32, i64, Box<Behavior<A>>),
    Limiter(i32, i64, i32, Box<Behavior<A>>),
    Repeat(i32, i32, Box<Behavior<A>>),
    ImmediateRepeat(i32, i32, Box<Behavior<A>>),
    Retry(i32, i32, Box<Behavior<A>>),
    ImmediateRetry(i32, i32, Box<Behavior<A>>),
    Log(i32, String, Box<Behavior<A>>),
    SubTree(i32, String, HashMap<String, String>),
}
