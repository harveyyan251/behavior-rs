use behavior::factory::{BtFactory, BtInstance};
use criterion::{criterion_group, criterion_main, Criterion};
use std::pin::Pin;

#[derive(Debug, Default)]
pub struct Context {}

#[derive(Debug, Default)]
pub struct World(pub u64);

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Entity(pub u64);

#[inline]
fn create_instance(bench_tree_json_str: &str) -> Pin<Box<BtInstance<Context, World, Entity>>> {
    let mut bt_factory = BtFactory::<Context, World, Entity>::new();
    bt_factory
        .compile_tree_template_from_json_str("bench_tree", bench_tree_json_str)
        .unwrap();
    bt_factory.create_tree_instance("bench_tree").unwrap()
}

fn select_node_with_one_child(c: &mut Criterion) {
    let bench_tree_json_str = r#"
    {
        "tree_blackboard": [
        ],
        "tree_structure": {
            "Select": [
                1,
                [
                    {
                        "AlwaysSuccess": 2
                    }
                ]
            ]
        }
    }"#;

    let mut instance = create_instance(bench_tree_json_str);
    c.bench_function("select_node_with_one_child", |b| {
        b.iter(|| instance.as_mut().tick(&mut World(0), &Entity(0)))
    });
}

fn select_node_with_ten_children(c: &mut Criterion) {
    let bench_tree_json_str = r#"
    {
        "tree_blackboard": [
        ],
        "tree_structure": {
            "Select": [
                1,
                [
                    {
                        "AlwaysFailure": 2
                    },
                    {
                        "AlwaysFailure": 3
                    },
                    {
                        "AlwaysFailure": 4
                    },
                    {
                        "AlwaysFailure": 5
                    },
                    {
                        "AlwaysFailure": 6
                    },
                    {
                        "AlwaysFailure": 7
                    },
                    {
                        "AlwaysFailure": 8
                    },
                    {
                        "AlwaysFailure": 9
                    },
                    {
                        "AlwaysFailure": 10
                    },
                    {
                        "AlwaysSuccess": 11
                    }
                ]
            ]
        }
    }"#;

    let mut instance = create_instance(bench_tree_json_str);
    c.bench_function("select_node_with_ten_children", |b| {
        b.iter(|| instance.as_mut().tick(&mut World(0), &Entity(0)))
    });
}

fn sequence_node_with_one_child(c: &mut Criterion) {
    let bench_tree_json_str = r#"
    {
        "tree_blackboard": [
        ],
        "tree_structure": {
            "Sequence": [
                1,
                [
                    {
                        "AlwaysSuccess": 2
                    }
                ]
            ]
        }
    }"#;

    let mut instance = create_instance(bench_tree_json_str);
    c.bench_function("sequence_node_with_one_child", |b| {
        b.iter(|| instance.as_mut().tick(&mut World(0), &Entity(0)))
    });
}

fn sequence_node_with_ten_children(c: &mut Criterion) {
    let bench_tree_json_str = r#"
    {
        "tree_blackboard": [
        ],
        "tree_structure": {
            "Sequence": [
                1,
                [
                    {
                        "AlwaysSuccess": 2
                    },
                    {
                        "AlwaysSuccess": 3
                    },
                    {
                        "AlwaysSuccess": 4
                    },
                    {
                        "AlwaysSuccess": 5
                    },
                    {
                        "AlwaysSuccess": 6
                    },
                    {
                        "AlwaysSuccess": 7
                    },
                    {
                        "AlwaysSuccess": 8
                    },
                    {
                        "AlwaysSuccess": 9
                    },
                    {
                        "AlwaysSuccess": 10
                    },
                    {
                        "AlwaysFailure": 11
                    }
                ]
            ]
        }
    }"#;

    let mut instance = create_instance(bench_tree_json_str);
    c.bench_function("sequence_node_with_ten_children", |b| {
        b.iter(|| instance.as_mut().tick(&mut World(0), &Entity(0)))
    });
}

criterion_group!(
    control_node_benches,
    select_node_with_one_child,
    select_node_with_ten_children,
    sequence_node_with_one_child,
    sequence_node_with_ten_children
);

criterion_main!(control_node_benches);
