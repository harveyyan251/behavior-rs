use behavior::{
    factory::{BtFactory, BtInstance},
    node::{BlackBoardCell, BtNode, BtNodeGenerator, DynamicCell, Executor, MetaDataCell},
    BehaviorError, BlackBoardMap, Status, TreeNodeBase,
};
use behavior::{generate_node, TreeNodeStatus};
use behavior_macros::TreeNodeStatus;
use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::hint::black_box;
use std::pin::Pin;

#[derive(Debug, Default)]
pub struct Context {}

#[derive(Debug, Default)]
pub struct World(pub u64);

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Entity(pub u64);

#[derive(Debug, TreeNodeStatus)]
pub struct BtActNodeExample {
    base: TreeNodeBase,
    bb_data1: BlackBoardCell<i32>,
    bb_data2: BlackBoardCell<f32>,
    meta_data1: MetaDataCell<i32>,
    meta_data2: MetaDataCell<f32>,
    dyn_data1: DynamicCell<i32>,
    dyn_data2: DynamicCell<f32>,
    tick_count: usize,
}
impl BtNodeGenerator for BtActNodeExample {
    type Context = Context;
    type World = World;
    type Entity = Entity;
    fn generate_node(
        tree_name: &str,
        tree_index: i32,
        tree_depth: i32,
        node_name: &str,
        node_index: i32,
        bb_map: &BlackBoardMap,
        meta_map: Option<&HashMap<String, String>>,
        bb_ref_map: Option<&HashMap<String, String>>,
        dyn_ref_map: Option<&HashMap<String, String>>,
    ) -> Result<Executor<Self::Context, Self::World, Self::Entity>, BehaviorError> {
        let tick_count = 0;
        generate_node!(
            tree_name, tree_index, tree_depth, node_name, node_index;
            bb_map, meta_map, bb_ref_map, dyn_ref_map;
            bb_data1, bb_data2;
            meta_data1, meta_data2;
            dyn_data1, dyn_data2;
            tick_count
        )
    }
}

impl BtNode for BtActNodeExample {
    type Context = Context;
    type World = World;
    type Entity = Entity;
    fn tick(&mut self, _ctx: &mut Context, _world: &mut World, _entity: &Entity) -> Status {
        // 下列运算逻辑开销大约为 0.7 ~ 0.8 ns
        self.tick_count.wrapping_add(1);
        *self.bb_data1 = self.meta_data1.get();
        *self.bb_data2 = self.meta_data2.get();
        if self.dyn_data1.is_mutable() {
            *self.dyn_data1 = *self.bb_data1;
        } else {
            *self.bb_data1 = self.dyn_data1.get();
        }
        if self.dyn_data2.is_mutable() {
            *self.dyn_data2 = *self.bb_data2;
        } else {
            *self.bb_data2 = self.dyn_data2.get();
        }
        Status::Success
    }
}

#[inline]
fn create_instance(bench_tree_json_str: &str) -> Pin<Box<BtInstance<Context, World, Entity>>> {
    let mut bt_factory = BtFactory::<Context, World, Entity>::new();
    bt_factory
        .compile_tree_template_from_json_str("bench_tree", bench_tree_json_str)
        .unwrap();
    bt_factory.create_tree_instance("bench_tree").unwrap()
}

fn action(c: &mut Criterion) {
    let bench_tree_json_str = r#"
    {
        "tree_blackboard": [
            {
                "bb_name": "blackboard_data1",
                "bb_type": "i32",
                "bb_value": "1000"
            },
            {
                "bb_name": "blackboard_data2",
                "bb_type": "f32",
                "bb_value": "2000.0"
            },
            {
                "bb_name": "blackboard_data3",
                "bb_type": "f32",
                "bb_value": "3000.0"
            }
        ],
        "tree_structure": {
            "Action": [
                1,
                {
                    "name": "BtActNodeExample",
                    "meta_map": {
                        "meta_data1": "10000",
                        "meta_data2": "20000.0"
                    },
                    "bb_ref_map": {
                        "bb_data1": "blackboard_data1",
                        "bb_data2": "blackboard_data2"
                    },
                    "dyn_ref_map": {
                        "dyn_data1": "11111",
                        "dyn_data2": "<blackboard_data3>"
                    }
                }
            ]
        }
    }"#;
    let mut bt_factory = BtFactory::<Context, World, Entity>::new();
    bt_factory.register_tree_node::<BtActNodeExample>();
    bt_factory
        .compile_tree_template_from_json_str("bench_tree", bench_tree_json_str)
        .unwrap();
    let mut instance = bt_factory.create_tree_instance("bench_tree").unwrap();
    c.bench_function("action", |b| {
        b.iter(|| instance.as_mut().tick(&mut World(0), &Entity(0)))
    });
}

fn empty_action(c: &mut Criterion) {
    let bench_tree_json_str = r#"
    {
        "tree_blackboard": [
            {
                "bb_name": "blackboard_data1",
                "bb_type": "i32",
                "bb_value": "1000"
            },
            {
                "bb_name": "blackboard_data2",
                "bb_type": "f32",
                "bb_value": "2000.0"
            },
            {
                "bb_name": "blackboard_data3",
                "bb_type": "f32",
                "bb_value": "3000.0"
            }
        ],
        "tree_structure": {
            "Action": [
                1,
                {
                    "name": "BtActNodeExample",
                    "meta_map": {
                        "meta_data1": "10000",
                        "meta_data2": "20000.0"
                    },
                    "bb_ref_map": {
                        "bb_data1": "blackboard_data1",
                        "bb_data2": "blackboard_data2"
                    },
                    "dyn_ref_map": {
                        "dyn_data1": "11111",
                        "dyn_data2": "<blackboard_data3>"
                    }
                }
            ]
        }
    }"#;
    let mut bt_factory = BtFactory::<Context, World, Entity>::new();
    bt_factory.register_tree_node::<BtActNodeExample>();
    bt_factory
        .compile_tree_template_from_json_str("bench_tree", bench_tree_json_str)
        .unwrap();
    let mut instance = bt_factory.create_tree_instance("bench_tree").unwrap();
    // 虚函数开销有大约 1.2 ~ 1.3 ns
    c.bench_function("empty_action", |b| {
        b.iter(|| {
            instance.as_mut().customized_tick(
                &mut World(0),
                &Entity(0),
                &mut |_task, _blackboard, _world, _entity| Status::Success,
            )
        })
    });
}

fn always_success(c: &mut Criterion) {
    let bench_tree_json_str = r#"
    {
        "tree_blackboard": [
        ],
        "tree_structure": {
            "AlwaysSuccess": 1
        }
    }"#;

    let mut instance = create_instance(bench_tree_json_str);
    c.bench_function("always_success", |b| {
        b.iter(|| instance.as_mut().tick(&mut World(0), &Entity(0)))
    });
}

fn always_failure(c: &mut Criterion) {
    let bench_tree_json_str = r#"
    {
        "tree_blackboard": [
        ],
        "tree_structure": {
            "AlwaysFailure": 1
        }
    }"#;

    let mut instance = create_instance(bench_tree_json_str);
    c.bench_function("always_failure", |b| {
        b.iter(|| instance.as_mut().tick(&mut World(0), &Entity(0)))
    });
}

fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}
pub fn fibonacci_bench(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
    // c.bench_function("fib 20", |b| b.iter(|| fibonacci(20)));
}

criterion_group!(
    leaf_node_benches,
    action,
    empty_action,
    always_success,
    always_failure,
    fibonacci_bench
);
criterion_main!(leaf_node_benches);
