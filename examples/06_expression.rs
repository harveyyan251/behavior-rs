use behavior::factory::BtFactory;

#[derive(Debug, Default)]
pub struct Context {}

#[derive(Debug, Default)]
pub struct World(pub u64);

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Entity(pub u64);

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

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

    // 优先级分支
    // let expression_json_str = r#"
    // {
    //     "tree_blackboard": [
    //         {
    //             "bb_name": "num",
    //             "bb_type": "i32",
    //             "bb_value": "0"
    //         }
    //     ],
    //     "tree_structure": {
    //         "PriorityBranch": [
    //             1,
    //             false,
    //             "100|75|50",
    //             {
    //                 "BranchCond": [
    //                     2,
    //                     [
    //                         {
    //                             "Expression": [
    //                                 3,
    //                                 "num < 10"
    //                             ]
    //                         },
    //                         {
    //                             "Expression": [
    //                                 4,
    //                                 "num < 20"
    //                             ]
    //                         },
    //                         {
    //                             "AlwaysSuccess": 5
    //                         }
    //                     ]
    //                 ]
    //             },
    //             [
    //                 {
    //                     "Expression": [
    //                         6,
    //                         "num += 2;"
    //                     ]
    //                 },
    //                 {
    //                     "Expression": [
    //                         7,
    //                         "num += 5"
    //                     ]
    //                 },
    //                 {
    //                     "Expression": [
    //                         8,
    //                         "num += 10"
    //                     ]
    //                 }
    //             ]
    //         ]
    //     }
    // }"#;

    let expression_json_str = r#"
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

    let entity = Entity(0);
    let mut world = World(0);

    let mut bt_factory = BtFactory::<Context, World, Entity>::new();
    bt_factory
        .compile_tree_template_from_json_str("expression", expression_json_str)
        .unwrap();

    let mut instance = bt_factory.create_tree_instance("expression").unwrap();
    for _ in 0..8 {
        // thread_sleep(Duration::from_millis(1000));
        instance.as_mut().tick(&mut world, &entity);
        tracing::info!(
            "blackboard: \n{}",
            instance.as_ref().visualize_blackboard_map().unwrap()
        );
    }
    tracing::info!(
        "tree: \n{}",
        instance.as_ref().visualize_tree_state().unwrap()
    );
}
