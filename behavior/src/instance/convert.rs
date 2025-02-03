use std::collections::{LinkedList, VecDeque};

pub trait ConvertFromStr: Sized {
    fn convert_from_str(s: &str) -> Option<Self>;
}

/// The following code has a Conflicting Implementation problem due to the Rust compiler.
// impl<T: std::str::FromStr> ConvertFromStr for T {
//     fn convert_from_str(s: &str) -> Option<Self> {
//         s.parse::<T>().ok()
//     }
// }

macro_rules! impl_convert_for_base_type {
    ($($base_type:ty),*) => {
        $(
            impl ConvertFromStr for $base_type {
                fn convert_from_str(s: &str) -> Option<Self> {
                    if s == "None" {
                        Some(<$base_type>::default())
                    } else {
                        s.parse::<$base_type>().ok()
                    }
                }
            }
        )*
    }
}

macro_rules! impl_convert_for_container {
    ($($container:ident),*) => {
        $(
            impl<T: ConvertFromStr> ConvertFromStr for $container<T> {
                fn convert_from_str(s: &str) -> Option<Self> {
                    if s == "None" {
                        Some($container::default())
                    } else {
                        s.split('|').map(|item| T::convert_from_str(item)).collect::<Option<$container<T>>>()
                    }
                }
            }
        )*
    }
}

impl<T: ConvertFromStr> ConvertFromStr for Option<T> {
    fn convert_from_str(s: &str) -> Option<Self> {
        if s == "None" {
            Some(Option::<T>::default())
        } else {
            match T::convert_from_str(s) {
                Some(v) => Some(Some(v)),
                None => None,
            }
        }
    }
}

// Based on my experience, the following base types is sufficient for blackboard
impl_convert_for_base_type!(bool, i32, i64, f32, f64, usize);
// impl_convert_from_str!(bool, usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, f32, f64);

impl_convert_for_container!(Vec, VecDeque, LinkedList);
// TODO: impl ConvertFromStr for HashSet, HashMap and BinaryHeap ?

#[cfg(test)]
mod tests {
    use super::ConvertFromStr;
    use std::collections::VecDeque;

    #[test]
    fn convert() {
        // bool
        assert_eq!(bool::convert_from_str("None"), Some(false));
        assert_eq!(bool::convert_from_str("true"), Some(true));
        assert_eq!(bool::convert_from_str("false"), Some(false));
        assert_eq!(bool::convert_from_str(""), None);
        assert_eq!(bool::convert_from_str("123"), None);

        // i32
        assert_eq!(i32::convert_from_str("None"), Some(i32::default()));
        assert_eq!(i32::convert_from_str("10"), Some(10));
        assert_eq!(i32::convert_from_str("10.0"), None);
        assert_eq!(i32::convert_from_str("-10"), Some(-10));
        assert_eq!(i32::convert_from_str("-10.0"), None);
        assert_eq!(i32::convert_from_str(""), None);
        assert_eq!(i32::convert_from_str("2200000000"), None);

        // f32
        assert_eq!(f32::convert_from_str("None"), Some(f32::default()));
        assert_eq!(f32::convert_from_str("10"), Some(10.0));
        assert_eq!(f32::convert_from_str("10.0"), Some(10.0));
        assert_eq!(f32::convert_from_str("-10"), Some(-10.0));
        assert_eq!(f32::convert_from_str("-10.0"), Some(-10.0));
        assert_eq!(f32::convert_from_str(""), None);
        assert_eq!(f32::convert_from_str("2200000000"), Some(2200000000.0));

        // Vec<i32>
        assert_eq!(Vec::<i32>::convert_from_str("None"), Some(Vec::new()));
        assert_eq!(Vec::<i32>::convert_from_str("1|2|3"), Some(vec![1, 2, 3]));
        assert_eq!(Vec::<i32>::convert_from_str("1|2|"), None);
        assert_eq!(Vec::<i32>::convert_from_str("1|2|3.0"), None);
        assert_eq!(Vec::<i32>::convert_from_str(""), None);

        // VecDeque<f32>
        assert_eq!(
            VecDeque::<f32>::convert_from_str("None"),
            Some(VecDeque::new())
        );
        assert_eq!(
            VecDeque::<f32>::convert_from_str("1.0|2|3.0"),
            Some(vec![1.0, 2.0, 3.0].into())
        );
        assert_eq!(Vec::<f32>::convert_from_str("1|2.0|"), None);
        assert_eq!(
            VecDeque::<f32>::convert_from_str("1|2|3.0"),
            Some(vec![1.0, 2.0, 3.0].into())
        );
        assert_eq!(Vec::<f32>::convert_from_str(""), None);

        // TODO: fix VecDeque<Vec<i32>>
    }
}
