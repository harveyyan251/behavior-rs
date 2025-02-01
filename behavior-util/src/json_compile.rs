// TODO: 重构该文件

use bincode::{
    config::{Configuration, Fixint, LittleEndian, NoLimit},
    encode_into_std_write, Decode, Encode,
};
use std::{io::Write, path::Path};
use tracing::error;

#[derive(Decode)]
pub struct Test {}

pub type BincodeConfig = Configuration<LittleEndian, Fixint, NoLimit>;
static CONFIG: BincodeConfig = bincode::config::standard()
    .with_no_limit()
    .with_little_endian()
    .with_fixed_int_encoding();

pub fn is_json_file(file_path: &Path) -> bool {
    file_path.exists()
        && file_path.is_file()
        && file_path.extension().map_or(false, |s| s == "json")
}

pub fn compile_json_to_bincode<T>(json_file_path: &Path) -> Result<(), ()>
where
    T: serde::de::DeserializeOwned + Encode + Decode,
{
    if !is_json_file(json_file_path) {
        error!(
            "compile_json_to_bincode error, file is not json file, json_file_path={:?}",
            json_file_path.display()
        );
        return Err(());
    }

    // debug!("json_file_path={}", json_file_path.display());
    let json_file = match std::fs::File::open(json_file_path) {
        Err(e) => {
            error!(
                "complie_json_to_bincode::open json file failed, json_file_path={}, error={:?}",
                json_file_path.display(),
                e
            );
            return Err(());
        }
        Ok(file) => file,
    };

    let reader = std::io::BufReader::new(json_file);
    let decode_value: T = match serde_json::from_reader(reader) {
        Err(e) => {
            error!(
                "complie_json_to_bincode::deserialize file failed!, json_file_path={}, error={:?}",
                json_file_path.display(),
                e
            );
            return Err(());
        }
        Ok(decode_value) => decode_value,
    };
    // debug!("deserialize json time={:?}", start_time.elapsed());

    let bincode_file_path = json_file_path.with_extension("bytes");
    let bincode_file_path = bincode_file_path.as_path();
    let bincode_file = match std::fs::File::create(bincode_file_path) {
        Err(e) => {
            error!(
                "compile::create file failed!, bincode_file_path={}, error={:?}",
                bincode_file_path.display(),
                e
            );
            return Err(());
        }
        Ok(file) => file,
    };
    let mut writer = std::io::BufWriter::new(bincode_file);
    let size = encode_into_std_write(&decode_value, &mut writer, CONFIG.clone()).unwrap();
    // debug!(
    //     "compile bincode time={:?}, size={}",
    //     start_time.elapsed(),
    //     size
    // );

    writer.flush().unwrap();
    let _ = deserialize_bincode_to_struct::<T>(bincode_file_path);
    Ok(())
}

pub fn deserialize_bincode_to_struct<T>(bincode_file_path: &Path) -> Result<T, ()>
where
    T: serde::de::DeserializeOwned + Encode + Decode,
{
    let bincode_file = match std::fs::File::open(bincode_file_path) {
        Err(e) => {
            error!(
                "deserialize_bincode_to_struct::open bincode file failed, bincode_file_path={}, error={:?}",
                bincode_file_path.display(),
                e
            );
            return Err(());
        }
        Ok(file) => file,
    };

    let mut reader = std::io::BufReader::new(bincode_file);
    let decode_value: T = match bincode::decode_from_std_read(&mut reader, CONFIG.clone()) {
        Err(e) => {
            error!(
                "deserialize_bincode_to_struct::from_reader deserialize file failed!, bincode_file_path={}, error={:?}",
                bincode_file_path.display(),
                e
            );
            return Err(());
        }
        Ok(decode_value) => decode_value,
    };
    // debug!(
    //     "deserialize bincode time={:?}, file_path={:?}\n",
    //     start_time.elapsed(),
    //     bincode_file_path.display()
    // );
    Ok(decode_value)
}

pub fn deserialize_config_to_struct<T>(json_file_path: &Path) -> Result<T, ()>
where
    T: serde::de::DeserializeOwned + Encode + Decode,
{
    if !is_json_file(json_file_path) {
        error!(
            "deserialize_config_to_struct error, file is not json file, json_file_path={:?}",
            json_file_path.display()
        );
        return Err(());
    }

    let bincode_file_path = json_file_path.with_extension("bytes");
    if bincode_file_path.exists() {
        return deserialize_bincode_to_struct::<T>(bincode_file_path.as_path());
    }

    let json_file = match std::fs::File::open(json_file_path) {
        Err(e) => {
            error!(
                "deserialize_json_to_struct::open json file failed, json_file_path={}, error={:?}",
                json_file_path.display(),
                e
            );
            return Err(());
        }
        Ok(file) => file,
    };

    let reader = std::io::BufReader::new(json_file);
    let decode_value: T = match serde_json::from_reader(reader) {
        Err(e) => {
            error!(
                "deserialize_json_to_struct::deserialize file failed!, json_file_path={}, error={:?}",
                json_file_path.display(),
                e
            );
            return Err(());
        }
        Ok(decode_value) => decode_value,
    };

    // debug!(
    //     "deserialize json time={:?}, file={:?}",
    //     start_time.elapsed(),
    //     json_file_path.display()
    // );
    Ok(decode_value)
}
