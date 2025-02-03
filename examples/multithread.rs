use std::thread::sleep;

use behavior::factory::BtFactory;

#[derive(Debug, Default)]
pub struct Context {}

#[derive(Debug, Default)]
pub struct World(pub u64);

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Entity(pub u64);

fn main() {
    // 普通分支
    let expression_json_str = r#"
    {
        "tree_blackboard": [
            {
                "bb_name": "num",
                "bb_type": "i32",
                "bb_value": "0"
            }
        ],
        "tree_structure": {
            "Branch": [
                1,
                false,
                {
                    "BranchCond": [
                        2,
                        [
                            {
                                "Expression": [
                                    3,
                                    "num < 10"
                                ]
                            },
                            {
                                "Expression": [
                                    4,
                                    "num < 20"
                                ]
                            },
                            {
                                "AlwaysSuccess": 5
                            }
                        ]
                    ]
                },
                [
                    {
                        "Expression": [
                            6,
                            "num += 2;"
                        ]
                    },
                    {
                        "Expression": [
                            7,
                            "num += 5"
                        ]
                    },
                    {
                        "Expression": [
                            8,
                            "num += 10"
                        ]
                    }
                ]
            ]
        }
    }"#;

    let _guard = ftlog::builder()
        .max_log_level(ftlog::LevelFilter::Info)
        // .time_format(time_format)
        // .bounded(100_1000, false)
        // .root(
        //     FileAppender::builder()
        //         .path("./ftlog.log")
        //         .rotate(Period::Day)
        //         .expire(time::Duration::days(7))
        //         .build(),
        // )
        .try_init()
        .unwrap();

    let entity = Entity(0);
    let mut world = World(0);

    let mut bt_factory = BtFactory::<Context, World, Entity>::new();
    bt_factory
        .compile_tree_template_from_json_str("expression", expression_json_str)
        .unwrap();

    let mut instance = bt_factory.create_tree_instance("expression").unwrap();
    // cargo run --release --package examples --example multithread --features behavior/gen_editor_data --features behavior/expression_node --features behavior/tree_visualization
    for _ in 0..2 {
        instance.as_mut().tick(&mut world, &entity);
        ftlog::info!(
            "blackboard:\n{}",
            instance.as_ref().visualize_blackboard_map().unwrap()
        );
    }

    // let (tx, rx) = crossbeam::channel::unbounded();
    let handle = std::thread::Builder::new()
        .name("tick".to_string())
        .spawn(move || {
            for _ in 2..6 {
                sleep(std::time::Duration::from_millis(1000));
                instance.as_mut().tick(&mut world, &entity);
                ftlog::info!(
                    "blackboard: \n{}",
                    instance.as_ref().visualize_blackboard_map().unwrap()
                );
            }
            (instance, world, entity)
        })
        .unwrap();
    let (mut instance, mut world, entity) = handle.join().unwrap();
    for _ in 6..8 {
        instance.as_mut().tick(&mut world, &entity);
        ftlog::info!(
            "blackboard: \n{}",
            instance.as_ref().visualize_blackboard_map().unwrap()
        );
    }
    ftlog::info!(
        "tree: \n{}",
        instance.as_ref().visualize_tree_state().unwrap()
    );
}
