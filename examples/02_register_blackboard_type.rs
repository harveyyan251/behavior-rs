use behavior::{
    convert::ConvertFromStr,
    factory::BtFactory,
    node::{BlackBoardCell, BtNode, BtNodeGenerator, DynamicCell, Executor, MetaDataCell},
    BehaviorError, BlackBoardMap, Status, TreeNode, TreeNodeBase,
};
use behavior::{generate_node, TreeNodeStatus};
use behavior_macros::TreeNodeStatus;
use ftlog::appender::*;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Default)]
pub struct Context {}

#[derive(Debug, Default)]
pub struct World(pub u64);

#[derive(Debug, Default, Clone)]
pub struct Entity(pub u64);

#[derive(Debug, TreeNodeStatus)]
pub struct BtActNodeExample {
    base: TreeNodeBase,
    bb_data1: BlackBoardCell<Option<Entity>>,
    bb_data2: BlackBoardCell<VecDeque<usize>>,
    bb_data3: BlackBoardCell<i32>,
    bb_data4: BlackBoardCell<f32>,
    meta_data1: MetaDataCell<i32>,
    meta_data2: MetaDataCell<f32>,
    dyn_data1: DynamicCell<i32>,
    dyn_data2: DynamicCell<i32>,
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
            bb_data1, bb_data2, bb_data3, bb_data4;
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
        self.tick_count += 1;
        if self.bb_data1.is_none() {
            *self.bb_data1 = Some(Entity(100));
        } else {
            self.bb_data1.unwrap_mut().0 += 100;
        }
        ftlog::info!("bb_data1={:?}", self.bb_data1.as_ref());

        if self.bb_data2.len() < 10 {
            self.bb_data2.push_back(self.tick_count);
        }
        ftlog::info!("bb_data2={:?}", self.bb_data2.as_ref());

        *self.bb_data3 += self.meta_data1.get();
        *self.bb_data4 += self.meta_data2.get();
        ftlog::info!("meta_data1={:?}", self.meta_data1.get());
        ftlog::info!("meta_data2={:?}", self.meta_data2.as_ref());
        ftlog::info!("bb_data3={:?}", self.bb_data3.as_ref());
        ftlog::info!("bb_data4={:?}", self.bb_data4.as_mut());

        if self.dyn_data1.is_mutable() {
            *self.dyn_data1 += 1111;
        }
        if self.dyn_data2.is_mutable() {
            *self.dyn_data2 += 1111;
        }
        ftlog::info!("dyn_data1={:?}", self.dyn_data1.as_ref());
        ftlog::info!("dyn_data2={:?}", self.dyn_data2.as_mut());

        Status::Success
    }
}

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

fn main() {
    let register_blackboard_json_str = r#"
    {
        "tree_blackboard": [
            {
                "bb_name": "blackboard_data1",
                "bb_type": "Option<Entity>",
                "bb_value": "None"
            },
            {
                "bb_name": "blackboard_data2",
                "bb_type": "VecDeque<usize>",
                "bb_value": "None"
            },
            {
                "bb_name": "blackboard_data3",
                "bb_type": "i32",
                "bb_value": "0"
            },
            {
                "bb_name": "blackboard_data4",
                "bb_type": "f32",
                "bb_value": "0"
            },
            {
                "bb_name": "blackboard_data5",
                "bb_type": "i32",
                "bb_value": "0"
            }
        ],
        "tree_structure": {
            "Select": [
                1,
                [
                    {
                        "Action": [
                            2,
                            {
                                "name": "BtActNodeExample",
                                "meta_map": {
                                    "meta_data1": "10",
                                    "meta_data2": "0.1"
                                },
                                "bb_ref_map": {
                                    "bb_data1": "blackboard_data1",
                                    "bb_data2": "blackboard_data2",
                                    "bb_data3": "blackboard_data3",
                                    "bb_data4": "blackboard_data4"
                                },
                                "dyn_ref_map": {
                                    "dyn_data1": "1111",
                                    "dyn_data2": "<blackboard_data5>"
                                }
                            }
                        ]
                    }
                ]
            ]
        }
    }"#;

    let _guard = ftlog::builder()
        .max_log_level(ftlog::LevelFilter::Info)
        .root(ChainAppenders::new(vec![
            Box::new(std::io::stdout()),
            Box::new(
                FileAppender::builder()
                    .path("./examples/log/02_register_blackboard_type.log")
                    .build(),
            ),
        ]))
        .try_init()
        .unwrap();

    let entity = Entity(0);
    let mut world = World(0);

    let mut bt_factory = BtFactory::<Context, World, Entity>::new();
    bt_factory.register_tree_node::<BtActNodeExample>();
    bt_factory.register_blackboard_type::<Option<Entity>>();
    bt_factory.register_blackboard_type::<VecDeque<usize>>();

    bt_factory
        .compile_tree_template_from_json_str("register_blackboard", register_blackboard_json_str)
        .unwrap();

    let mut instance = bt_factory
        .create_tree_instance("register_blackboard")
        .unwrap();

    for _ in 0..10 {
        instance.as_mut().customized_tick(
            &mut world,
            &entity,
            &mut |task, blackboard, world, entity| {
                let start_time = std::time::Instant::now();
                let status = task.action_tick(blackboard.context_mut(), world, &entity);
                ftlog::info!("elapsed={:?}", start_time.elapsed());
                status
            },
        );
    }
    ftlog::info!(
        "tree: \n{}",
        instance.as_ref().visualize_tree_state().unwrap()
    );
    ftlog::info!(
        "blackboard: \n{}",
        instance.as_ref().visualize_blackboard_map().unwrap()
    );
    ftlog::info!("{:?}", instance.as_mut().context_mut());
}
