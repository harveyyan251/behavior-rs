#[allow(warnings)]
mod json_compile;
mod random;
use ahash::{HashMapExt, RandomState};
pub use random::*;
use regex::Regex;
use std::{
    collections::HashMap,
    sync::{LazyLock, OnceLock, RwLock},
};

pub fn type_name<T: ?Sized>() -> &'static str {
    let raw_type_name: &str = std::any::type_name::<T>();
    type CacheMap = HashMap<&'static str, &'static str, RandomState>;
    static CACHE: LazyLock<RwLock<CacheMap>> = LazyLock::new(|| RwLock::new(CacheMap::new()));
    if let Some(type_name) = CACHE.read().unwrap().get(raw_type_name) {
        return type_name;
    }
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let type_name = Box::new(
        REGEX
            .get_or_init(|| Regex::new(r"[_a-zA-Z0-9]+?::").unwrap())
            .replace_all(raw_type_name, "")
            .into_owned(),
    );
    let type_name = Box::leak(type_name.into_boxed_str());
    CACHE.write().unwrap().insert(raw_type_name, type_name);
    type_name
}

pub fn simplified_name<T: ?Sized>() -> &'static str {
    let type_name = type_name::<T>();
    type_name.split('<').next().unwrap_or(&"")
}

#[cfg(test)]
mod tests {
    use crate::simplified_name;

    use super::type_name;
    use ahash::RandomState;
    use std::collections::{HashMap, HashSet, VecDeque};

    #[test]
    fn test_type_name() {
        assert_eq!(type_name::<i32>(), "i32");
        assert_eq!(simplified_name::<i32>(), "i32");
        assert_eq!(type_name::<String>(), "String");
        assert_eq!(simplified_name::<String>(), "String");
        assert_eq!(type_name::<&str>(), "&str");
        assert_eq!(simplified_name::<&str>(), "&str");
        // assert_eq!(
        //     type_name::<HashMap<&'static str, &'static str, RandomState>>(),
        //     "HashMap<&'static str, &'static str, RandomState>",
        //     "std::any::type_name={}",
        //     std::any::type_name::<HashMap<&'static str, &'static str, RandomState>>(),
        // );
        assert_eq!(
            type_name::<HashMap<&'static str, &'static str, RandomState>>(),
            "HashMap<&str, &str, RandomState>",
        );
        assert_eq!(
            simplified_name::<HashMap<&'static str, &'static str, RandomState>>(),
            "HashMap",
        );
        assert_eq!(
            type_name::<
                HashMap<String, HashMap<i32, VecDeque<HashSet<i32>>, RandomState>, RandomState>,
            >(),
            "HashMap<String, HashMap<i32, VecDeque<HashSet<i32>>, RandomState>, RandomState>",
        );
        assert_eq!(
            simplified_name::<
                HashMap<String, HashMap<i32, VecDeque<HashSet<i32>>, RandomState>, RandomState>,
            >(),
            "HashMap",
        );
    }
}
