use behavior::{
    convert::ConvertFromStr,
    factory::BtFactory,
    node::{BlackBoardCell, BtNode, BtNodeGenerator, Executor, MetaDataCell},
    BehaviorError, BlackBoardMap, EditorNodeDataGenerator, Status, TreeNodeBase,
};
use behavior::{generate_node, TreeNodeStatus};
use behavior_macros::{EditorNodeDataGenerator, TreeNodeStatus};
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

#[derive(Debug, TreeNodeStatus, EditorNodeDataGenerator)]
pub struct BtActNodeExample {
    base: TreeNodeBase,
    bb_data1: BlackBoardCell<Option<Entity>>,
    bb_data2: BlackBoardCell<VecDeque<i32>>,
    bb_data3: BlackBoardCell<i32>,
    bb_data4: BlackBoardCell<f32>,
    meta_data1: MetaDataCell<f32>,
    meta_data2: MetaDataCell<i32>,
    meta_data3: MetaDataCell<i32>,
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
            meta_data1, meta_data2, meta_data3;
            ;
            node_data1, node_data2
        )
    }
}

impl BtNode for BtActNodeExample {
    type Context = Context;
    type World = World;
    type Entity = Entity;
    fn tick(&mut self, _ctx: &mut Context, _world: &mut World, _entity: &Entity) -> Status {
        if self.bb_data1.is_none() {
            *self.bb_data1 = Some(Entity(self.meta_data2.get() as u64));
            self.bb_data1
                .set(Some(Entity(self.meta_data3.get() as u64)));
            let target_entity = self.bb_data1.unwrap_mut();
            *target_entity = Entity(300);
            if self.bb_data2.is_empty() {
                self.bb_data2.push_back(self.meta_data1.get() as i32);
            }
        }
        // (*self.bb_data2).push_back(100);
        *self.bb_data3 += 1;
        *self.bb_data4 += 1.0;
        self.node_data1 += 10.0;
        self.node_data2 += 10;
        Status::Success
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let example_tree_json = r#"
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
    bt_factory.register_blackboard_type::<Option<Entity>>();
    bt_factory.register_blackboard_type::<VecDeque<i32>>();
    bt_factory
        .compile_tree_template_from_json_str("test1", example_tree_json)
        .unwrap();

    let mut instance = bt_factory.create_tree_instance("test1").unwrap();
    // TODO: 正式的性能测试
    // cargo run --release --package examples --example 03_simple_stress --features behavior/gen_editor_data --features behavior/expression_node
    let start_time = std::time::Instant::now();
    for _ in 0..10_000_000 {
        instance.as_mut().tick(&mut world, &entity);
    }
    tracing::info!("elapsed={:?}", start_time.elapsed());

    let mut num: usize = 0;
    let mut queue = VecDeque::new();
    let start_time = std::time::Instant::now();
    for i in 0..10_000_000 {
        num = num.wrapping_add(i) % 10_1000;
        // num = num.wrapping_add(i) % std::hint::black_box(10_000);
        queue.push_back(num);
    }
    tracing::info!("elapsed={:?}", start_time.elapsed());
    tracing::info!(
        "blackboard: \n{}",
        instance.as_ref().visualize_blackboard_map().unwrap()
    );
}
