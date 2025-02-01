# Behavior Tree for Rust

## Overview

This project provides a behavior tree implementation in Rust. The implementation includes various nodes that can be utilized to build complex AI behaviors through a simple and understandable tree structure. As of now, the project development is approximately 70% complete.

## Action Node Definition

An example of defining an ActionNode is shown below:

```rust

#[derive(Debug, Default)]
pub struct Context {}

#[derive(Debug, Default)]
pub struct World(pub u64);

#[derive(Debug, Default)]
pub struct Entity(pub u64);

#[derive(Debug, TreeNodeStatus)]
pub struct BtActNodeExample {
    base: TreeNodeBase,
    bb_data1: BlackBoardCell<Option<Entity>>,
    bb_data2: BlackBoardCell<VecDeque<i32>>,
    bb_data3: BlackBoardCell<i32>,
    bb_data4: BlackBoardCell<f32>,
    meta_data1: MetaDataCell<f32>,
    meta_data2: MetaDataCell<i32>,
    meta_data3: MetaDataCell<i32>,
    meta_data4: MetaDataCell<i32>,
    dyn_data1: DynamicCell<i32>,
    dyn_data2: DynamicCell<i32>,
    node_data1: f32,
    node_data2: i32,
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
        let node_data1 = 10.0;
        let node_data2 = 20;
        generate_node!(
            tree_name, tree_index, tree_depth, node_name, node_index;
            bb_map, meta_map, bb_ref_map, dyn_ref_map;
            bb_data1, bb_data2, bb_data3, bb_data4;
            meta_data1, meta_data2, meta_data3, meta_data4;
            dyn_data1, dyn_data2;
            node_data1, node_data2
        )
    }
}

impl BtNode for BtActNodeExample {
    type Context = Context;
    type World = World;
    type Entity = Entity;

    fn tick(&mut self, _ctx: &mut Context, _world: &mut World, _entity: &Entity) -> Status {
        tracing::info!("-----------------------------(BtNodeExample::tick start)-----------------------------");
        let start_time = std::time::Instant::now();

        if self.bb_data1.is_some() {
            tracing::info!("bb_data1={:?}", self.bb_data1.unwrap_ref());
        } else {
            *self.bb_data1 = Some(Entity(100));
            tracing::info!("after_set::bb_data1={:?}", self.bb_data1.unwrap_ref());
            *self.bb_data1.as_mut() = Some(Entity(200));
            tracing::info!("after_first_modify::bb_data1={:?}", self.bb_data1.unwrap_ref());

            let target_entity = self.bb_data1.unwrap_mut();
            *target_entity = Entity(300);
            tracing::info!("after_second_modify::bb_data1={:?}", self.bb_data1.unwrap_value());
        }

        (*self.bb_data2).push_back(100);
        tracing::info!("bb_data2={:?}", self.bb_data2.as_ref());
        tracing::info!("meta_data1={}", self.meta_data1.as_ref());
        tracing::info!("meta_data2={}", self.meta_data2.get());
        tracing::info!("meta_data3={}", *self.meta_data3);
        tracing::info!("meta_data4={}", *self.meta_data4);

        *self.bb_data3 += 1;
        *self.bb_data4 += 1.0;
        tracing::info!("bb_data3={}", self.bb_data3.as_ref());
        tracing::info!("bb_data4={}", self.bb_data4.as_ref());
        tracing::info!("dyn_data1={}", self.dyn_data1.as_ref());
        tracing::info!("dyn_data2={}", self.dyn_data2.as_ref());

        self.node_data1 += 10.0;
        self.node_data2 += 10;
        tracing::info!("node_data1={}", self.node_data1);
        tracing::info!("node_data2={}", self.node_data2);
        tracing::info!("elapsed={:?}", start_time.elapsed());

        Status::Success
    }
}
```

## Using a Tree Instance

To create and tick a behavior tree instance, follow this example:

```rust

let basic_tree_json_str = r#"
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
        }
    ],
    "tree_structure": {
        "Sequence": [
            1,
            [
                {
                    "Action": [
                        2,
                        {
                            "name": "BtActNodeExample",
                            "meta_map": {
                                "meta_data1": "10000",
                                "meta_data2": "20000"
                            },
                            "bb_ref_map": {
                                "bb_data1": "blackboard_data1",
                                "bb_data2": "blackboard_data2"
                            },
                            "dyn_ref_map": {
                                "dyn_data1": "11111",
                                "dyn_data2": "<blackboard_data2>"
                            }
                        }
                    ]
                },
                {
                    "Action": [
                        3,
                        {
                            "name": "BtActNodeExample",
                            "meta_map": {
                                "meta_data1": "50000",
                                "meta_data2": "10000"
                            },
                            "bb_ref_map": {
                                "bb_data1": "blackboard_data1",
                                "bb_data2": "blackboard_data2"
                            },
                            "dyn_ref_map": {
                                "dyn_data1": "2222",
                                "dyn_data2": "<blackboard_data2>"
                            }
                        }
                    ]
                }
            ]
        ]
    }
}"#;

let entity = Entity(0);
let mut world = World(0);

let mut bt_factory = BtFactory::<Context, World, Entity>::new();
bt_factory.register_tree_node::<BtActNodeExample>();
bt_factory
    .compile_tree_template_from_json_str("basic_tree", basic_tree_json_str)
    .unwrap();

let mut instance = bt_factory.create_tree_instance("basic_tree").unwrap();
// instance.as_mut().tick(&mut world, &entity);
    instance.as_mut().customized_tick(
        &mut world,
        &entity,
        &mut |task, blackboard, world, entity| {
            let start_time = std::time::Instant::now();
            let status = task.action_tick(blackboard.context_mut(), world, &entity);
            tracing::info!("elapsed={:?}", start_time.elapsed());
            status
        },
    );
```

## Example

See [Example](https://github.com/harveyyan251/behavior-rs/tree/master/examples) folder.

## Summary

This behavior tree framework is designed to provide a flexible and efficient way to implement AI behaviors in Rust. However, please note that the project is still under development, with approximately 70% of the planned features already implemented. Contributions and suggestions for improvements are welcome!
