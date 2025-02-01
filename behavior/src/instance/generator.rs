// TODO: 删除该无用文件
#[cfg(test)]
mod test {
    //     #[test]
    //     fn test_bt_weight_select() {
    //         LogConfig::new()
    //             .enable_console()
    //             .with_console_color(true)
    //             .build();

    //         let weight_select_tree_json = r#"
    // {
    //     "blackboard": [
    //         {
    //             "name": "num",
    //             "ty": "i32",
    //             "value": "0"
    //         }
    //     ],
    //     "nodes": {
    //         "WeightSelect": [
    //             1,
    //             "10|6|2",
    //             [
    //                 {
    //                     "Debug": [
    //                         2,
    //                         "num",
    //                         {
    //                             "Expression": [
    //                                 3,
    //                                 "num += 2;"
    //                             ]
    //                         }
    //                     ]
    //                 },
    //                 {
    //                     "Debug": [
    //                         4,
    //                         "num",
    //                         {
    //                             "Expression": [
    //                                 5,
    //                                 "num += 3;"
    //                             ]
    //                         }
    //                     ]
    //                 },
    //                 {
    //                     "Debug": [
    //                         6,
    //                         "num",
    //                         {
    //                             "Expression": [
    //                                 7,
    //                                 "num +=5;"
    //                             ]
    //                         }
    //                     ]
    //                 }
    //             ]
    //         ]
    //     }
    // }"#;

    //         let e1 = Entity(1);
    //         let mut bt_manager = BTManager::new();
    //         bt_manager.compile_bt_tree("BtEditor1".to_string(), weight_select_tree_json);
    //         bt_manager.instantiate_bt(e1, "BtEditor1".to_string());

    //         let dt = 0.1;
    //         let mut world = CellWorld::new(0);

    //         for _ in (0..10) {
    //             bt_manager.tick(dt, &mut world);
    //         }
    //     }

    //     #[test]
    //     fn test_gen_editor_import_data() {
    //         LogConfig::new()
    //             .enable_console()
    //             .with_console_color(true)
    //             .build();

    //         let tree_name = "test";
    //         let node_name = "BtActNodeExample";
    //         let bb_map: BlackBoardMap = HashMap::new();
    //         let bb_ref_map: Option<&HashMap<String, String>> = None;
    //         let dyn_ref_map: Option<&HashMap<String, String>> = None;
    //         let meta_map: Option<&HashMap<String, String>> = None;
    //         let mut serde_info = Some(vec![]);
    //         get_node_executor(
    //             tree_name,
    //             node_name,
    //             &bb_map,
    //             meta_map,
    //             bb_ref_map,
    //             dyn_ref_map,
    //             &mut serde_info,
    //         );
    //         let enums = get_editor_enum();
    //         let value = serde_json::json!({
    //             "enums": enums,
    //             "nodes": serde_info,
    //         });
    //         // println!("{:?}", value.to_string());
    //         match std::fs::OpenOptions::new()
    //             .write(true)
    //             .create(true)
    //             .truncate(true)
    //             .open("target/newnodedef.json")
    //         {
    //             Ok(mut file) => {
    //                 std::io::Write::write_all(&mut file, value.to_string().as_bytes()).unwrap();
    //             }
    //             Err(e) => {
    //                 use std::io::ErrorKind;
    //                 let current_dir = std::env::current_dir().expect("Failed to get current directory");
    //                 eprintln!("Current directory: {:?}", current_dir);

    //                 match e.kind() {
    //                     ErrorKind::NotFound => eprintln!("Error: File not found (code = 2)"),
    //                     ErrorKind::PermissionDenied => {
    //                         eprintln!("Error: Permission denied (code = 13)")
    //                     }
    //                     ErrorKind::AlreadyExists => eprintln!("Error: File already exists (code = 17)"),
    //                     _ => eprintln!(
    //                         "Failed to open file: {:?} (code = {:?}",
    //                         e,
    //                         e.raw_os_error().unwrap_or(-1)
    //                     ),
    //                 }
    //                 // panic!("Failed to open file: {:?}", e); // 确保程序中止
    //             }
    //         }
    //     }
}
