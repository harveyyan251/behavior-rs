use behavior::{
    convert::ConvertFromStr,
    factory::BtFactory,
    node::{BlackBoardCell, BtNode, BtNodeGenerator, DynamicCell, Executor, MetaDataCell},
    BehaviorError, BlackBoardMap, Status, TreeNode, TreeNodeBase,
};
use behavior::{generate_node, TreeNodeStatus};
use behavior_macros::TreeNodeStatus;
use std::collections::{HashMap, VecDeque};

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
        tracing::info!(
            "-----------------------------(BtNodeExample::tick start)-----------------------------"
        );
        let start_time = std::time::Instant::now();
        if self.bb_data1.is_some() {
            tracing::info!("bb_data1={:?}", self.bb_data1.unwrap_ref());
        } else {
            *self.bb_data1 = Some(Entity(100));
            tracing::info!("after_set::bb_data1={:?}", self.bb_data1.unwrap_ref());
            *self.bb_data1.as_mut() = Some(Entity(200));
            tracing::info!(
                "after_first_modify::bb_data1={:?}",
                self.bb_data1.unwrap_ref()
            );
            let target_entity = self.bb_data1.unwrap_mut();
            *target_entity = Entity(300);
            tracing::info!(
                "after_second_modify::bb_data1={:?}",
                self.bb_data1.unwrap_value()
            );
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
        // 这里节点 elapsed 时间开销都在 tracing::info! 上
        tracing::info!("elapsed={:?}", start_time.elapsed());
        Status::Success
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

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
                "bb_type": "VecDeque<i32>",
                "bb_value": "None"
            },
            {
                "bb_name": "blackboard_data3",
                "bb_type": "i32",
                "bb_value": "0"
            },
            {
                "bb_name": "blackboard_data3",
                "bb_type": "i32",
                "bb_value": "0"
            },
            {
                "bb_name": "blackboard_data4",
                "bb_type": "f32",
                "bb_value": "0.0"
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
                                    "meta_data1": "10.0",
                                    "meta_data2": "20",
                                    "meta_data3": "30",
                                    "meta_data4": "1"
                                },
                                "bb_ref_map": {
                                    "bb_data1": "blackboard_data1",
                                    "bb_data2": "blackboard_data2",
                                    "bb_data3": "blackboard_data3",
                                    "bb_data4": "blackboard_data4"
                                },
                                "dyn_ref_map": {
                                    "dyn_data1": "10",
                                    "dyn_data2": "<blackboard_data3>"
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
    // 注册自定义的黑板值类型
    bt_factory.register_blackboard_type::<Option<Entity>>();
    bt_factory.register_blackboard_type::<VecDeque<i32>>();
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
                tracing::info!("elapsed={:?}", start_time.elapsed());
                status
            },
        );
    }
    tracing::info!(
        "tree: \n{}",
        instance.as_ref().visualize_tree_state().unwrap()
    );
    tracing::info!(
        "blackboard: \n{}",
        instance.as_ref().visualize_blackboard_map().unwrap()
    );
    tracing::info!("{:?}", instance.as_mut().context_mut());
}
