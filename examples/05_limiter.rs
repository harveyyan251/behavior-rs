use behavior::{
    convert::ConvertFromStr,
    factory::BtFactory,
    node::{BlackBoardCell, BtNode, BtNodeGenerator, Executor},
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

impl ConvertFromStr for Entity {
    fn convert_from_str(s: &str) -> Option<Self> {
        if s == "None" {
            Some(Entity::default())
        } else {
            match s.parse::<u64>() {
                Ok(v) => Some(Entity(v)),
                Err(_) => None,
            }
        }
    }
}

#[derive(Debug, TreeNodeStatus)]
pub struct BtActNodeExample {
    base: TreeNodeBase,
    bb_data1: BlackBoardCell<i32>,
    bb_data2: BlackBoardCell<f32>,
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
        // 解决需要导入 get_metadata, get_blackboard 和 get_dynamic 函数的问题
        generate_node!(
            tree_name, tree_index, tree_depth, node_name, node_index;
            bb_map, meta_map, bb_ref_map, dyn_ref_map;
            bb_data1, bb_data2;
            ;
            ;
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
        *self.bb_data1 += 10;
        ftlog::info!("bb_data1={}", *self.bb_data1);
        *self.bb_data2 += 10.0;
        ftlog::info!("bb_data2={}", *self.bb_data2);
        Status::Success
    }
}

fn main() {
    let limiter_json_str = r#"
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
            "Limiter": [
                1,
                5000,
                3,
                {
                    "Action": [
                        2,
                        {
                            "name": "BtActNodeExample",
                            "meta_map": {
                            },
                            "bb_ref_map": {
                                "bb_data1": "blackboard_data1",
                                "bb_data2": "blackboard_data2"
                            },
                            "dyn_ref_map": {
                            }
                        }
                    ]
                }
            ]
        }
    }"#;

    let log_path = std::path::Path::new("./examples/log/05_limiter.log");
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
        .compile_tree_template_from_json_str("limiter", limiter_json_str)
        .unwrap();

    let mut instance = bt_factory.create_tree_instance("limiter").unwrap();
    for _ in 0..10 {
        // thread_sleep(std::time::Duration::from_millis(1000));
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
