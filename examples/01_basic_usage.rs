use behavior::{
    factory::BtFactory,
    node::{BlackBoardCell, BtNode, BtNodeGenerator, DynamicCell, Executor, MetaDataCell},
    BehaviorError, BlackBoardMap, Status, TreeNodeBase,
};
use behavior::{generate_node, TreeNodeStatus};
use behavior_macros::TreeNodeStatus;
use ftlog::appender::*;
use std::collections::HashMap;

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
    tick_count: i32,
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
        ftlog::info!(
            "-----------------------------(BtNodeExample::tick start)-----------------------------"
        );
        self.tick_count = self.tick_count.wrapping_add(1);
        *self.bb_data1 += self.meta_data1.get();
        *self.bb_data2 += self.meta_data2.get();
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
        ftlog::info!("bb_data1={}", self.bb_data1.get());
        ftlog::info!("bb_data2={}", self.bb_data2.get());
        ftlog::info!("meta_data1={}", self.meta_data1.get());
        ftlog::info!("meta_data2={}", self.meta_data2.get());
        ftlog::info!("dyn_data1={}", self.dyn_data1.get());
        ftlog::info!("dyn_data2={}", self.dyn_data2.get());
        ftlog::info!("tick_count={}", self.tick_count);
        Status::Success
    }
}

fn main() {
    let basic_tree_json_str = r#"
    {
        "tree_blackboard": [
            {
                "bb_name": "blackboard_data1",
                "bb_type": "i32",
                "bb_value": "0"
            },
            {
                "bb_name": "blackboard_data2",
                "bb_type": "f32",
                "bb_value": "0.0"
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
                                    "dyn_data1": "111",
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
                                    "meta_data1": "100000",
                                    "meta_data2": "200000"
                                },
                                "bb_ref_map": {
                                    "bb_data1": "blackboard_data1",
                                    "bb_data2": "blackboard_data2"
                                },
                                "dyn_ref_map": {
                                    "dyn_data1": "222",
                                    "dyn_data2": "<blackboard_data2>"
                                }
                            }
                        ]
                    }
                ]
            ]
        }
    }"#;

    let log_path = std::path::Path::new("./examples/log/01_basic_usage.log");
    std::fs::remove_file(log_path).unwrap();
    let _guard = ftlog::builder()
        .max_log_level(ftlog::LevelFilter::Info)
        .root(ChainAppenders::new(vec![
            Box::new(std::io::stdout()),
            Box::new(FileAppender::builder().path(log_path).build()),
        ]))
        .try_init()
        .unwrap();

    let entity = Entity(0);
    let mut world = World(0);

    let mut bt_factory = BtFactory::<Context, World, Entity>::new();
    bt_factory.register_tree_node::<BtActNodeExample>();
    bt_factory
        .compile_tree_template_from_json_str("basic_tree", basic_tree_json_str)
        .unwrap();

    let mut instance = bt_factory.create_tree_instance("basic_tree").unwrap();
    ftlog::info!("tree_size={}", std::mem::size_of_val(&instance));
    for _ in 0..2 {
        instance.as_mut().tick(&mut world, &entity);
    }

    ftlog::info!(
        "tree: \n{}",
        instance.as_ref().visualize_tree_state().unwrap()
    );
    ftlog::info!(
        "blackboard: \n{}",
        instance.as_ref().visualize_blackboard_map().unwrap()
    );
}
