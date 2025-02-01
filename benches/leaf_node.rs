use behavior::factory::{BtFactory, BtInstance};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
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
    always_success,
    always_failure,
    fibonacci_bench
);
criterion_main!(leaf_node_benches);
