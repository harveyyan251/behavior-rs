// TODO: 重构该文件
#[allow(warnings)]
use glam::{Vec2, Vec3};
use rand::seq::index::{self};
use rand::{
    distributions::{
        uniform::{SampleRange, SampleUniform},
        Distribution, Standard, WeightedIndex,
    },
    seq::SliceRandom,
    Rng,
};
use rand_distr::{num_traits::Float, uniform::SampleBorrow, Normal, StandardNormal};
use std::fmt::Debug;

/// commonly used random functions encapsulated from rand library to prevent unexpected panics.

#[inline]
// 随机值
pub fn random_value<T>() -> T
where
    Standard: Distribution<T>,
{
    // random value in Type Range for Integer such as i32, u32, i64, u64
    // random value in [0.0, 1.0) for f32 and f64
    // random true or false for bool
    // random character for char
    rand::thread_rng().gen()
}

#[inline]
// 范围内随机值
pub fn random_value_in_range<T, R>(range: R) -> T
where
    T: Default + SampleUniform,
    R: Debug + SampleRange<T>,
{
    if range.is_empty() {
        T::default()
    } else {
        rand::thread_rng().gen_range(range)
    }
}

#[inline]
// 按概率随机测试
pub fn random_pass<F: Float>(probability: F) -> bool {
    match probability {
        p if !p.is_normal() || p.is_sign_negative() => false,
        p if p >= F::one() => true,
        p => p.to_f64().map_or(false, |p| rand::thread_rng().gen_bool(p)),
    }
}

#[inline]
// 按比率随机测试
pub fn random_pass_by_ratio(numerator: u32, denominator: u32) -> bool {
    if denominator == 0 {
        false
    } else if numerator > denominator {
        true
    } else {
        rand::thread_rng().gen_ratio(numerator, denominator)
    }
}

#[inline]
// 随机字符
pub fn random_char() -> char {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789";
    CHARSET[rand::thread_rng().gen_range(0..CHARSET.len())] as char
}

#[inline]
// 随机字符串
pub fn random_string(len: usize) -> String {
    (0..len).map(|_| random_char()).collect()
}

#[inline]
// 随机洗牌
pub fn random_shuffle<T>(items: &mut [T]) {
    items.shuffle(&mut rand::thread_rng())
}

#[inline]
// 正态分布, 均值为 mean, 标准差为 std_dev
pub fn random_normal<F>(mean: F, std_dev: F) -> F
where
    F: Float + Default,
    StandardNormal: Distribution<F>,
{
    match Normal::new(mean, std_dev) {
        Ok(normal) => normal.sample(&mut rand::thread_rng()),
        Err(_) => F::default(),
    }
}

#[inline]
// 概率归一化
pub fn normalize<F: Float>(probabilities: &mut [F]) -> bool {
    let sum = probabilities.iter().try_fold(F::zero(), |accum, &prob| {
        if prob.is_nan() || prob.is_infinite() || prob.is_sign_negative() || prob.is_subnormal() {
            return None;
        }
        Some(accum + prob)
    });
    if let Some(sum) = sum.filter(|sum| sum.is_normal()) {
        probabilities
            .iter_mut()
            .for_each(|prob| *prob = *prob / sum);
        return true;
    }
    false
}

#[inline]
// 矩形内随机点
pub fn random_point_in_rect<F: Float + SampleUniform>(
    pos: Vec3,
    half_length: F,
    half_width: F,
) -> Vec3 {
    if (half_length.is_normal() || half_length.is_zero())
        && (half_width.is_normal() || half_width.is_zero())
        && half_length.is_sign_positive()
        && half_width.is_sign_positive()
    {
        let x_offset = rand::thread_rng()
            .gen_range(-half_length..=half_length)
            .to_f32()
            .unwrap_or(0.0);
        let z_offset = rand::thread_rng()
            .gen_range(-half_width..=half_width)
            .to_f32()
            .unwrap_or(0.0);
        return Vec3::new(pos.x + x_offset, pos.y, pos.z + z_offset);
    }
    pos
}

#[inline]
// 矩形上随机点
pub fn random_point_on_rect<F: Float + SampleUniform>(
    pos: Vec3,
    half_width: F,
    half_height: F,
) -> Vec3 {
    if half_width.is_normal()
        && half_height.is_normal()
        && half_width.is_sign_positive()
        && half_height.is_sign_positive()
    {
        let two = F::from(2.0).unwrap();
        let width = two * half_width;
        let heigh = two * half_height;
        // expand the sides of the rectangle into line to randomly generate points.
        let line = two * (width + heigh);
        let num = rand::thread_rng().gen_range(F::zero()..line);
        let x_offset = match num {
            num if num < width => num - half_width,
            num if num < width + heigh => half_width,
            num if num < width + heigh + width => half_width - (num - width - heigh),
            _ => -half_width,
        }
        .to_f32()
        .unwrap_or(0.0);
        let z_offset = match num {
            num if num < width => half_height,
            num if num < width + heigh => half_height - (num - width),
            num if num < width + heigh + width => -half_height,
            _ => -half_height + (num - width - heigh - width),
        }
        .to_f32()
        .unwrap_or(0.0);
        return Vec3::new(pos.x + x_offset, pos.y, pos.z + z_offset);
    }
    pos
}

#[inline]
// 正方形内随机点
pub fn random_point_in_square<F: Float + SampleUniform>(pos: Vec3, half_side: F) -> Vec3 {
    random_point_in_rect(pos, half_side, half_side)
}

#[inline]
// 正方形上随机点
pub fn random_point_on_square<F: Float + SampleUniform>(pos: Vec3, half_side: F) -> Vec3 {
    random_point_on_rect(pos, half_side, half_side)
}

// #[inline]
// 长方体内随机点
// pub fn random_point_in_cuboid<F: Float + SampleUniform>(
//     pos: Vec3,
//     rotation: Vec3,
//     half_length: F,
//     half_width: F,
//     half_height: F,
// ) -> Vec3 {
//     let random_pos = if half_length.is_normal()
//         && half_width.is_normal()
//         && half_height.is_normal()
//         && half_length.is_sign_positive()
//         && half_width.is_sign_positive()
//         && half_height.is_sign_positive()
//     {
//         let rng = &mut rand::thread_rng();
//         let x_offset = rng
//             .gen_range(-half_width..=half_width)
//             .to_f32()
//             .unwrap_or(0.0);
//         let y_offset = rng
//             .gen_range(-half_height..=half_height)
//             .to_f32()
//             .unwrap_or(0.0);
//         let z_offset = rng
//             .gen_range(-half_length..=half_length)
//             .to_f32()
//             .unwrap_or(0.0);
//         Vec3::new(pos.x + x_offset, pos.y + y_offset, pos.z + z_offset)
//     } else {
//         pos
//     };
//     math::apply_euler_angles_to_vector(&(random_pos - pos), &rotation) + pos
// }

// // 正方体内随机点
// pub fn random_point_in_cube<F: Float + SampleUniform>(
//     pos: Vec3,
//     rotation: Vec3,
//     half_side: F,
// ) -> Vec3 {
//     random_point_in_cuboid(pos, rotation, half_side, half_side, half_side)
// }

#[inline]
// 环内随机点
pub fn random_point_in_annulus<F: Float + SampleUniform>(
    pos: Vec3,
    inner_radius: F,
    outer_radius: F,
) -> Vec3 {
    if inner_radius.is_normal()
        && outer_radius.is_normal()
        && inner_radius.is_sign_positive()
        && outer_radius.is_sign_positive()
        && inner_radius < outer_radius
    {
        let tau = F::from(2.0 * std::f64::consts::PI).unwrap();
        let mut rng = rand::thread_rng();
        // generates a random angle between [0, 2 * PI).
        let theta = rng.gen_range(F::zero()..tau);
        // generates a random radius within the range [inner_radius^2, outer_radius^2) and sqrt() to ensure an even distribution within the annulus.
        let r = (rng.gen_range(inner_radius * inner_radius..=outer_radius * outer_radius)).sqrt();
        let x_offset = (r * theta.cos()).to_f32().unwrap_or(0.0);
        let z_offset = (r * theta.sin()).to_f32().unwrap_or(0.0);
        return Vec3::new(pos.x + x_offset, pos.y, pos.z + z_offset);
    }
    pos
}

#[inline]
// 圆内随机点
pub fn random_point_in_circle<F: Float + SampleUniform>(pos: &Vec3, radius: F) -> Vec3 {
    // can't use random_point_in_annulus directly because it doesn't deal subnormal inner_radius such as zero.
    // random_point_in_annulus(pos, 0.0, radius)
    if radius.is_normal() && radius.is_sign_positive() {
        let tau = F::from(2.0 * std::f64::consts::PI).unwrap();
        let mut rng = rand::thread_rng();
        // generates a random angle between [0, 2 * PI).
        let theta = rng.gen_range(F::zero()..tau);
        // generates a random radius within the range [0, radius) and sqrt() is done to ensure an even distribution within the circle.
        let r = rng.gen_range::<F, _>(F::zero()..=F::one()).sqrt() * radius;
        let x_offset = (r * theta.cos()).to_f32().unwrap_or(0.0);
        let z_offset = (r * theta.sin()).to_f32().unwrap_or(0.0);
        return Vec3::new(pos.x + x_offset, pos.y, pos.z + z_offset);
    }
    pos.clone()
}

#[inline]
// 圆上随机点
pub fn random_point_on_circle<F: Float + SampleUniform>(pos: Vec3, radius: F) -> Vec3 {
    // use random_point_in_annulus directly will be slower
    // random_point_in_annulus(pos, radius, radius)
    if radius.is_normal() && radius.is_sign_positive() {
        let tau = F::from(2.0 * std::f64::consts::PI).unwrap();
        let mut rng = rand::thread_rng();
        let theta = rng.gen_range(F::zero()..tau);
        let x_offset = (radius * theta.cos()).to_f32().unwrap_or(0.0);
        let z_offset = (radius * theta.sin()).to_f32().unwrap_or(0.0);
        return Vec3::new(pos.x + x_offset, pos.y, pos.z + z_offset);
    }
    pos
}

#[inline]
// 球内随机点
pub fn random_point_in_sphere<F: Float + SampleUniform>(pos: &Vec3, radius: F) -> Vec3 {
    if radius.is_normal() && radius.is_sign_positive() {
        let pi = F::from(std::f64::consts::PI).unwrap();
        let tau = F::from(2.0 * std::f64::consts::PI).unwrap();
        let cb = F::from(1.0f64 / 3.0f64).unwrap();
        let mut rng = rand::thread_rng();

        let u = rng.gen_range(F::zero()..F::one());
        // generate two random angles, theta is the horizontal angle, phi is the vertical angle.
        let theta = rng.gen_range(F::zero()..tau); // θ in [0, 2π)
        let phi = rng.gen_range(F::zero()..pi); // φ in [0, π)

        // use the cube root to ensure uniform distribution.
        let r = radius * (u).powf(cb);

        // x_offset = r * sin(φ) * cos(θ), y_offset = r * sin(φ) * sin(θ), z_offset = r * cos(φ)
        let x_offset = (r * phi.sin() * theta.cos()).to_f32().unwrap_or(0.0);
        let y_offset = (r * phi.sin() * theta.sin()).to_f32().unwrap_or(0.0);
        let z_offset = (r * phi.cos()).to_f32().unwrap_or(0.0);
        return Vec3::new(pos.x + x_offset, pos.y + y_offset, pos.z + z_offset);
    }
    pos.clone()
}

// #[inline]
// // 圆柱内随机，TODO: 加旋转参数
// pub fn random_point_in_cylinder<F: Float + SampleUniform>(
//     pos: Vec3,
//     rotation: Vec3,
//     radius: F,
//     half_height: F,
// ) -> Vec3 {
//     let random_point = if radius.is_normal()
//         && radius.is_sign_positive()
//         && half_height.is_normal()
//         && half_height.is_sign_positive()
//     {
//         let tau = F::from(2.0 * std::f64::consts::PI).unwrap();
//         let mut rng = rand::thread_rng();
//         let theta = rng.gen_range(F::zero()..tau);

//         let r = rng.gen_range::<F, _>(F::zero()..=F::one()).sqrt() * radius;
//         let x_offset = (r * theta.cos()).to_f32().unwrap_or(0.0);
//         let z_offset = (r * theta.sin()).to_f32().unwrap_or(0.0);
//         let y_offset = rng
//             .gen_range(-half_height..half_height)
//             .to_f32()
//             .unwrap_or(0.0);
//         Vec3::new(pos.x + x_offset, pos.y + y_offset, pos.z + z_offset)
//     } else {
//         pos.clone()
//     };
//     // random_point
//     // println!("random_point1={}", random_point);
//     let random_point = math::apply_euler_angles_to_vector(&(random_point - pos), &rotation) + pos;
//     // println!("random_point2={}", random_point);
//     random_point
// }

#[inline]
// 等概率抽取一个下标
pub fn average_select_index(len: usize) -> usize {
    assert!(len > 0, "average_select_index, len must > 0");
    rand::thread_rng().gen_range(0..len)
}

#[inline]
// 等概率不重复抽取n个下标
pub fn average_select_indexs(len: usize, mut select_num: usize) -> Vec<usize> {
    // index::sample panics if select_num > len
    select_num = std::cmp::min(len, select_num);
    index::sample(&mut rand::thread_rng(), len, select_num).into_vec()
}

// #[inline]
// 旧做法
// 等概率不重复抽取n个下标
// pub fn _average_select_indexs(len: usize, select_num: usize) -> Vec<usize> {
//     if len == 0 || select_num == 0 {
//         Vec::default()
//     } else if select_num == 1 {
//         // use `average_select_one_index` instead.
//         vec![rand::thread_rng().gen_range(0..len)]
//     } else {
//         // n should not be too large, otherwise thread local will take up a lot of memory.
//         const WARN_INDEX_LEN: usize = 1_000_000;
//         if len > WARN_INDEX_LEN {
//             warn!("average_select_indexs::warn, len is too large, len={}", len);
//         }
//         // thread local cached indexs can avoid the performance overhead of index collect
//         thread_local! {
//             static INDEXS: std::cell::RefCell<Vec<usize>> = std::cell::RefCell::new(Vec::new());
//         }
//         INDEXS.with(|indexs| {
//             let mut indexs = indexs.borrow_mut();
//             let curr_len = indexs.len();
//             if curr_len < len {
//                 indexs.extend(curr_len..len);
//             }
//             // if select_num > len, choose_multiple will just choose len elements.
//             indexs[0..len]
//                 .choose_multiple(&mut rand::thread_rng(), select_num)
//                 .copied()
//                 .collect()
//         })
//     }
// }

#[inline]
// 等概率重复抽取n个下标
pub fn average_repeated_select_indexs(len: usize, select_num: usize) -> Vec<usize> {
    if len == 0 {
        Vec::new()
    } else {
        (0..select_num)
            .map(|_| rand::thread_rng().gen_range(0..len))
            .collect::<Vec<_>>()
    }
}

#[inline]
// 等概率抽取一项, 返回引用
pub fn average_select_item<T>(items: &[T]) -> &T {
    assert!(items.len() > 0, "average_select_item, items.len() must > 0");
    &items[rand::thread_rng().gen_range(0..items.len())]
}

#[inline]
// 等概率抽取一项, 返回值
pub fn average_collect_item<T: Clone>(items: &[T]) -> T {
    assert!(
        items.len() > 0,
        "average_collect_item, items.len() must > 0"
    );
    items[rand::thread_rng().gen_range(0..items.len())].clone()
}

// 等概率不重复抽取n项, 返回引用
#[inline]
pub fn average_select_items<T>(items: &[T], select_num: usize) -> Vec<&T> {
    items
        .choose_multiple(&mut rand::thread_rng(), select_num)
        .collect()
}

// 等概率不重复抽取n项, 返回值
#[inline]
pub fn average_collect_items<T: Clone>(items: &[T], collect_num: usize) -> Vec<T> {
    items
        .choose_multiple(&mut rand::thread_rng(), collect_num)
        .cloned()
        .collect()
}

#[inline]
// 等概率重复抽取n项, 返回引用
pub fn average_repeated_select_items<T>(items: &[T], select_num: usize) -> Vec<&T> {
    if items.is_empty() {
        Vec::new()
    } else {
        (0..select_num)
            .map(|_| &items[rand::thread_rng().gen_range(0..items.len())])
            .collect::<Vec<_>>()
    }
}

// 等概率重复抽取n项, 返回值
pub fn average_repeated_collect_items<T: Clone>(items: &[T], select_num: usize) -> Vec<T> {
    if items.is_empty() {
        Vec::new()
    } else {
        (0..select_num)
            .map(|_| items[rand::thread_rng().gen_range(0..items.len())].clone())
            .collect::<Vec<_>>()
    }
}

// #[inline]
// pub fn weight_select_one_index<I, X>(weights: I) -> Option<usize>
// where
//     I: IntoIterator,
//     I::Item: SampleBorrow<X>,
//     X: for<'a> ::core::ops::AddAssign<&'a X> + Clone + Default + SampleUniform + PartialOrd,
// {
//     // returns an error if the weights is empty, if any weight is `< 0`, or if its total value is 0.
//     WeightedIndex::new(weights)
//         .ok()
//         .map(|dist| dist.sample(&mut rand::thread_rng()))
// }

#[inline]
// 按权重抽取一个下标
pub fn weight_select_index<F, X>(weights: &[F]) -> Option<usize>
where
    F: Float + SampleBorrow<X>,
    for<'c> &'c F: SampleBorrow<X>,
    X: for<'a> ::core::ops::AddAssign<&'a X> + Clone + Default + SampleUniform + PartialOrd,
{
    // returns an error if the weights is empty, if any weight is `< 0`, or if its total value is 0.
    WeightedIndex::new(weights)
        .ok()
        .map(|dist| dist.sample(&mut rand::thread_rng()))
}

#[inline]
// 按权重比例不重复抽取n个下标
pub fn weight_select_indexs<F, X>(weights: &[F], select_num: usize) -> Option<Vec<usize>>
where
    F: Float + SampleBorrow<X>,
    for<'c> &'c F: SampleBorrow<X>,
    X: for<'a> ::core::ops::AddAssign<&'a X> + Clone + Default + SampleUniform + PartialOrd,
{
    WeightedIndex::new(weights).ok().map(|dist| {
        dist.sample_iter(&mut rand::thread_rng())
            .take(select_num)
            .collect()
    })
}

#[inline]
// 按权重比例重复抽取n个下标
pub fn weight_repeated_select_indexs<F, X>(weights: &[F], select_num: usize) -> Option<Vec<usize>>
where
    F: Float + SampleBorrow<X>,
    for<'c> &'c F: SampleBorrow<X>,
    X: for<'a> ::core::ops::AddAssign<&'a X> + Clone + Default + SampleUniform + PartialOrd,
{
    WeightedIndex::new(weights).ok().map(|dist| {
        (0..select_num)
            .map(|_| dist.sample(&mut rand::thread_rng()))
            .collect::<Vec<_>>()
    })
}

#[inline]
// 按权重比例抽取一项, 返回引用
pub fn weight_select_item<'a, 'b, T, F, X>(weights: &[F], items: &'b [T]) -> Option<&'b T>
where
    F: Float + SampleBorrow<X>,
    for<'c> &'c F: SampleBorrow<X>,
    X: for<'d> ::core::ops::AddAssign<&'d X> + Clone + Default + SampleUniform + PartialOrd,
{
    (weights.len() == items.len())
        .then(|| weight_select_index(weights).map(|index| &items[index]))
        .flatten()
}

#[inline]
// 按权重比例抽取一项, 返回值
pub fn weight_collect_item<T, F, X>(weights: &[F], items: &[T]) -> Option<T>
where
    T: Clone,
    F: Float + SampleBorrow<X>,
    for<'c> &'c F: SampleBorrow<X>,
    X: for<'d> ::core::ops::AddAssign<&'d X> + Clone + Default + SampleUniform + PartialOrd,
{
    (weights.len() == items.len())
        .then(|| weight_select_index(weights).map(|index| items[index].clone()))
        .flatten()
}

#[inline]
// 按权重比例不重复抽取n项, 返引用
pub fn weight_select_items<'a, 'b, T, F, X>(
    weights: &'a [F],
    items: &'b [T],
    select_num: usize,
) -> Option<Vec<&'b T>>
where
    F: Float + SampleBorrow<X>,
    for<'c> &'c F: SampleBorrow<X>,
    X: for<'d> ::core::ops::AddAssign<&'d X> + Clone + Default + SampleUniform + PartialOrd,
{
    (weights.len() == items.len())
        .then(|| {
            WeightedIndex::new(weights).ok().map(|dist| {
                dist.sample_iter(&mut rand::thread_rng())
                    .take(select_num)
                    .map(|index| &items[index])
                    .collect::<Vec<&'b T>>()
            })
        })
        .flatten()
}

#[inline]
// 按权重比例不重复抽取n项, 返回值
pub fn weight_collect_items<T, F, X>(
    weights: &[F],
    items: &[T],
    select_num: usize,
) -> Option<Vec<T>>
where
    T: Clone,
    F: Float + SampleBorrow<X>,
    for<'c> &'c F: SampleBorrow<X>,
    X: for<'d> ::core::ops::AddAssign<&'d X> + Clone + Default + SampleUniform + PartialOrd,
{
    (weights.len() == items.len())
        .then(|| {
            WeightedIndex::new(weights).ok().map(|dist| {
                dist.sample_iter(&mut rand::thread_rng())
                    .take(select_num)
                    .map(|index| items[index].clone())
                    .collect::<Vec<T>>()
            })
        })
        .flatten()
}

#[inline]
// 按权重比例重复抽取n项, 返引用
pub fn weight_repeated_select_items<'a, 'b, T, F, X>(
    weights: &'a [F],
    items: &'b [T],
    select_num: usize,
) -> Option<Vec<&'b T>>
where
    F: Float + SampleBorrow<X>,
    for<'c> &'c F: SampleBorrow<X>,
    X: for<'d> ::core::ops::AddAssign<&'d X> + Clone + Default + SampleUniform + PartialOrd,
{
    (weights.len() == items.len())
        .then(|| {
            WeightedIndex::new(weights).ok().map(|dist| {
                (0..select_num)
                    .map(|_| &items[dist.sample(&mut rand::thread_rng())])
                    .collect::<Vec<&'b T>>()
            })
        })
        .flatten()
}

#[inline]
// 按权重比例重复抽取n项, 返回值
pub fn weight_repeated_collect_items<T, F, X>(
    weights: &[F],
    items: &[T],
    select_num: usize,
) -> Option<Vec<T>>
where
    T: Clone,
    F: Float + SampleBorrow<X>,
    for<'c> &'c F: SampleBorrow<X>,
    X: for<'d> ::core::ops::AddAssign<&'d X> + Clone + Default + SampleUniform + PartialOrd,
{
    (weights.len() == items.len())
        .then(|| {
            WeightedIndex::new(weights).ok().map(|dist| {
                (0..select_num)
                    .map(|_| items[dist.sample(&mut rand::thread_rng())].clone())
                    .collect::<Vec<T>>()
            })
        })
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;
    use termplot::*;

    #[test]
    fn test_corner_case() {
        // in rust, 1..0 represents an inverted range, meaning the start value is greater than the end value.
        // typically, such a range is considered empty, the Rust standard library handles this situation explicitly.
        println!("{}", (1..0).is_empty());
        let mut items = Vec::<i32>::new();
        // shuffle handles this situation correctly.
        random_shuffle(items.as_mut_slice());

        // try choose items more than the length of the vec.
        let items = vec![1, 2, 3, 4, 5];
        let selected_items = items
            .choose_multiple(&mut rand::thread_rng(), 10)
            .collect::<Vec<&i32>>();
        println!("{:?}", selected_items);

        // subnormal float
        let float32: f32 = 1.0e-40;
        let float64: f64 = 1.0e-320;
        println!(
            "float32.is_subnormal()={}, float64.is_subnormal()={}",
            float32.is_subnormal(),
            float64.is_subnormal()
        );

        // zero is normal?
        println!(
            "0.0f32.is_normal()={}, 0.0f64.is_normal()={}",
            0.0f32.is_normal(),
            0.0f64.is_normal()
        )
    }

    #[test]
    fn test_random_value() {
        println!("random_i8={}", random_value::<i8>());
        println!("random_i16={}", random_value::<i16>());
        println!("random_i32={}", random_value::<i32>());
        println!("random_i64={}", random_value::<i64>());
        println!("random_i128={}", random_value::<i128>());
        println!("random_isize={}", random_value::<isize>());

        println!("random_u8={}", random_value::<u8>());
        println!("random_u16={}", random_value::<u16>());
        println!("random_u32={}", random_value::<u32>());
        println!("random_u64={}", random_value::<u64>());
        println!("random_u128={}", random_value::<u128>());
        println!("random_usize={}", random_value::<usize>());

        println!("random_f32={}", random_value::<f32>());
        println!("random_f64={}", random_value::<f64>());
        println!("random_bool={}", random_value::<bool>());
        println!("random_char={}", random_value::<char>());
    }

    #[test]
    fn test_random_value_in_range() {
        println!("random_int32={}", random_value_in_range(1..10));
        println!("random_int32={}", random_value_in_range(10..10));
        println!("random_float32={}", random_value_in_range(-12.0..24.0));
        println!("random_float32={}", random_value_in_range(12.0..12.0));
    }

    #[test]
    fn test_random_pass() {
        const TEST_TIMES: usize = 10_000_000;
        const TEST_RANGE: usize = 10;
        const INIT_PROBABILITY: f64 = 0.618f64 / TEST_RANGE as f64;
        const STEP_PROBABILITY: f64 = 1.0f64 / TEST_RANGE as f64;
        const EPS: f64 = 0.0001;
        let mut exceed_deviation_num = 0;
        let mut probabilities = vec![0.0f64; TEST_RANGE];
        probabilities
            .iter_mut()
            .scan(INIT_PROBABILITY, |state, prob| {
                *prob = *state;
                *state += STEP_PROBABILITY;
                Some(*prob)
            })
            .for_each(|theo_probability| {
                let mut pass_times = 0;
                for _ in 0..TEST_TIMES {
                    if random_pass(theo_probability) {
                        pass_times += 1;
                    }
                }
                let real_probability = pass_times as f64 / TEST_TIMES as f64;
                println!(
                    "test_times={}, pass_times={}, theo_probability={}, real_probability={}",
                    TEST_TIMES, pass_times, theo_probability, real_probability
                );
                let deviation = (real_probability - theo_probability).abs();
                if deviation > EPS {
                    exceed_deviation_num += 1;
                    println!("deviation={} > {}", deviation, EPS);
                }
            });
        println!("exceed_deviation_num={}", exceed_deviation_num);
    }

    #[test]
    fn test_random_shuffle() {
        const SHUFFLE_TIMES: usize = 10_000_000;
        const SHUFFLE_RANGE: usize = 10;
        const THEO_PROBABILITY: f64 = 1.0f64 / SHUFFLE_RANGE as f64;
        const EPS: f64 = 0.0001;
        let mut position_times = [[0; SHUFFLE_RANGE]; SHUFFLE_RANGE];
        let mut items = (0..SHUFFLE_RANGE).collect::<Vec<_>>();
        let cost = std::time::Instant::now();
        for _ in 0..SHUFFLE_TIMES {
            random_shuffle(items.as_mut_slice());
            items.iter().enumerate().for_each(|(index, &num)| {
                position_times[num][index] += 1;
            });
        }
        println!("cost={:?}", cost.elapsed());
        for num in 0..SHUFFLE_RANGE {
            let mut real_probabilities = position_times[num]
                .iter()
                .map(|num| *num as f64)
                .collect::<Vec<f64>>();
            normalize(real_probabilities.as_mut_slice());
            let mut exceed_deviation_num = 0;
            real_probabilities
                .iter()
                .enumerate()
                .for_each(|(index, &real_probability)| {
                    let deviation = (real_probability - THEO_PROBABILITY).abs();
                    println!(
                        "num={}, pos={}, theo_probability={}, real_probability={}",
                        num, index, THEO_PROBABILITY, real_probability
                    );
                    if deviation > EPS {
                        println!("deviation={} > {}", deviation, EPS);
                        exceed_deviation_num += 1;
                    }
                });
            println!("exceed_deviation_num={}", exceed_deviation_num);
        }
    }

    #[test]
    fn test_termplot() {
        let mut plot = Plot::default();
        plot.set_domain(Domain(-10.0..10.0))
            .set_codomain(Domain(-2.0..2.0))
            .set_title("Sin Func")
            .set_x_label("X axis")
            .set_y_label("Y axis")
            .set_size(Size::new(100, 25))
            .add_plot(Box::new(plot::Graph::new(|x| x.sin())));
        println!("{}", plot);
    }

    #[test]
    fn test_random_in_rect() {
        struct Rect {
            center: Vec3,
            half_width: f32,
            half_height: f32,
        }
        impl DrawView for Rect {
            fn draw(&self, _: &View, canvas: &mut ViewCanvas) {
                const POINT_NUM: usize = 10_000;
                for _ in 0..POINT_NUM {
                    let point =
                        random_point_in_rect(self.center, self.half_width, self.half_height);
                    let x = point.x as f64;
                    let y = point.z as f64;
                    canvas.point(x, y);
                }
            }
        }
        let rect = Rect {
            center: Vec3::new(0.0, 0.0, 0.0),
            half_width: 800.0,
            half_height: 400.0,
        };
        let mut plot = Plot::default();
        plot.set_domain(Domain(-1000.0..1000.0))
            .set_codomain(Domain(-1000.0..1000.0))
            .set_title("In Rect")
            .set_size(Size::new(100, 100))
            .add_plot(Box::new(rect));
        println!("{}", plot);
    }

    #[test]
    fn test_random_on_rect() {
        struct Rect {
            center: Vec3,
            half_width: f32,
            half_height: f32,
        }
        impl DrawView for Rect {
            fn draw(&self, _: &View, canvas: &mut ViewCanvas) {
                const POINT_NUM: usize = 1_000;
                for _ in 0..POINT_NUM {
                    let point =
                        random_point_on_rect(self.center, self.half_width, self.half_height);
                    let x = point.x as f64;
                    let y = point.z as f64;
                    canvas.point(x, y);
                }
            }
        }
        let rect = Rect {
            center: Vec3::new(0.0, 0.0, 0.0),
            half_width: 800.0,
            half_height: 400.0,
        };
        let mut plot = Plot::default();
        plot.set_domain(Domain(-1000.0..1000.0))
            .set_codomain(Domain(-500.0..500.0))
            .set_title("In Rect")
            .set_size(Size::new(100, 100))
            .add_plot(Box::new(rect));
        println!("{}", plot);
    }

    #[test]
    fn test_random_in_circle() {
        struct Circle {
            center: Vec3,
            radius: f32,
        }
        impl DrawView for Circle {
            fn draw(&self, _: &View, canvas: &mut ViewCanvas) {
                const POINT_NUM: usize = 10_000;
                for _ in 0..POINT_NUM {
                    let point = random_point_in_circle(&self.center, self.radius);
                    let x = point.x as f64;
                    let y = point.z as f64;
                    canvas.point(x, y);
                }
            }
        }
        let circle = Circle {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 800.0,
        };
        let mut plot = Plot::default();
        plot.set_domain(Domain(-1000.0..1000.0))
            .set_codomain(Domain(-1000.0..1000.0))
            .set_title("In Circle")
            .set_size(Size::new(100, 100))
            .add_plot(Box::new(circle));
        println!("{}", plot);
    }

    #[test]
    fn test_random_in_circle_by_sample() {
        struct Rect {
            bottom_left: Vec2,
            top_right: Vec2,
        }
        impl Rect {
            fn new(bottom_left: Vec2, top_right: Vec2) -> Self {
                Rect {
                    bottom_left,
                    top_right,
                }
            }
            fn in_rect(&self, point: Vec2) -> bool {
                point.x >= self.bottom_left.x
                    && point.y >= self.bottom_left.y
                    && point.x < self.top_right.x
                    && point.y < self.top_right.y
            }
        }
        const TEST_TIMES: usize = 20_000_000;
        const SAMPLE_RANGE: usize = 40;
        const SIDE_LENGHT: usize = 2;
        const RADIUS: f32 = 100.0;
        const THEO_PROBABILITY: f64 =
            (SIDE_LENGHT * SIDE_LENGHT) as f64 / (RADIUS * RADIUS * PI) as f64;
        const EPS: f64 = 0.0001;
        let mut sample_rects = Vec::new();
        let mut sample_hit_times = vec![0; SAMPLE_RANGE / SIDE_LENGHT];
        for num in (0..SAMPLE_RANGE).into_iter().step_by(SIDE_LENGHT) {
            let num = num as f32 - (SAMPLE_RANGE as f32 / 2.0);
            let bottom_left = Vec2::new(num, num);
            let top_left = Vec2::new(num + SIDE_LENGHT as f32, num + SIDE_LENGHT as f32);
            sample_rects.push(Rect::new(bottom_left, top_left));
        }
        let center = Vec3::new(0.0, 0.0, 0.0);
        let cost = std::time::Instant::now();
        for _ in 0..TEST_TIMES {
            let point = random_point_in_circle(&center, RADIUS);
            let point = Vec2::new(point.x, point.z);
            for index in 0..SAMPLE_RANGE / SIDE_LENGHT {
                if sample_rects[index].in_rect(point) {
                    sample_hit_times[index] += 1;
                    break;
                }
            }
        }
        println!("cost={:?}", cost.elapsed());
        let mut exceed_deviation_num = 0;
        for index in 0..SAMPLE_RANGE / SIDE_LENGHT {
            let real_probability = sample_hit_times[index] as f64 / TEST_TIMES as f64;
            println!(
                "sample_hit_times[{}]={}, theo_probability={}, real_probability={}",
                index, sample_hit_times[index], THEO_PROBABILITY, real_probability
            );
            let deviation = (real_probability - THEO_PROBABILITY).abs();
            if deviation > EPS {
                exceed_deviation_num += 1;
                println!("deviation={} > {}", deviation, EPS);
            }
        }
        println!("exceed_deviation_num={}", exceed_deviation_num);
    }

    #[test]
    fn test_random_on_circle() {
        struct Circle {
            center: Vec3,
            radius: f32,
        }
        impl DrawView for Circle {
            fn draw(&self, _: &View, canvas: &mut ViewCanvas) {
                const POINT_NUM: usize = 5_000;
                for _ in 0..POINT_NUM {
                    let point = random_point_on_circle(self.center, self.radius);
                    let x = point.x as f64;
                    let y = point.z as f64;
                    canvas.point(x, y);
                }
            }
        }
        let circle = Circle {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 800.0,
        };
        let mut plot = Plot::default();
        plot.set_domain(Domain(-1000.0..1000.0))
            .set_codomain(Domain(-1000.0..1000.0))
            .set_title("In Circle")
            .set_size(Size::new(100, 100))
            .add_plot(Box::new(circle));
        println!("{}", plot);
    }

    #[test]
    fn test_random_in_annulus() {
        struct Annulus {
            center: Vec3,
            inner_radius: f32,
            outer_radius: f32,
        }
        impl DrawView for Annulus {
            fn draw(&self, _: &View, canvas: &mut ViewCanvas) {
                const POINT_NUM: usize = 7_000;
                for _ in 0..POINT_NUM {
                    let point =
                        random_point_in_annulus(self.center, self.inner_radius, self.outer_radius);
                    let x = point.x as f64;
                    let y = point.z as f64;
                    canvas.point(x, y);
                }
            }
        }
        let annulus = Annulus {
            center: Vec3::new(0.0, 0.0, 0.0),
            inner_radius: 600.0,
            outer_radius: 800.0,
        };
        let mut plot = Plot::default();
        plot.set_domain(Domain(-1000.0..1000.0))
            .set_codomain(Domain(-1000.0..1000.0))
            .set_title("In Annulus")
            .set_size(Size::new(100, 100))
            .add_plot(Box::new(annulus));
        println!("{}", plot);
    }

    // #[test]
    // fn test_random_point_in_cuboid() {
    //     let center = Vec3::new(0.0, 0.0, 0.0);
    //     let rotation = Vec3::new(0.0, 0.0, 0.0);
    //     let half_length = 10.0;
    //     let half_width = 10.0;
    //     let half_height = 10.0;
    //     for _ in (0..10) {
    //         println!(
    //             "random_point={}",
    //             random_point_in_cuboid(center, rotation, half_length, half_width, half_height)
    //         )
    //     }
    // }

    // #[test]
    // fn test_random_point_in_cylinder() {
    //     let center = Vec3::new(0.0, 0.0, 0.0);
    //     let radius = 10.0;
    //     let half_height = 10.0;
    //     let rotation = Vec3::new(0.0, 180.0, 0.0);
    //     for _ in (0..10) {
    //         let random_point = random_point_in_cylinder(center, rotation, radius, half_height);
    //         let distance = (random_point.x.powi(2) + random_point.z.powi(2)).sqrt();
    //         // println!("random_point={}, distance={}", random_point, distance);
    //     }
    // }

    #[test]
    fn test_average_select_index() {
        const SELECT_TIMES: usize = 100_000_000;
        const SELECT_RANGE: usize = 1000;
        const THEO_PROBABILITY: f64 = 1.0f64 / SELECT_RANGE as f64;
        const EPS: f64 = 0.00001;
        let mut selected_times = [0; SELECT_RANGE];
        let cost = std::time::Instant::now();
        for _ in 0..SELECT_TIMES {
            selected_times[average_select_index(SELECT_RANGE)] += 1;
        }
        println!("cost={:?}", cost.elapsed());
        let mut exceed_deviation_num = 0;
        for index in 0..SELECT_RANGE {
            let real_probability = selected_times[index] as f64 / SELECT_TIMES as f64;
            println!(
                "selected_times[{}]={}, theo_probability={}, real_probability={}",
                index, selected_times[index], THEO_PROBABILITY, real_probability
            );
            let deviation = (real_probability - THEO_PROBABILITY).abs();
            if deviation > EPS {
                exceed_deviation_num += 1;
                println!("deviation={} > {}", deviation, EPS);
            }
        }
        println!("exceed_deviation_num={}", exceed_deviation_num);
    }

    #[test]
    fn test_average_select_indexs() {
        const SELECT_TIMES: usize = 10_000_000;
        const SELECT_RANGE: usize = 100;
        const SELECT_NUM: usize = 10;
        const ALL_SELECT_NUM: usize = SELECT_TIMES * SELECT_NUM;
        const THEO_PROBABILITY: f64 = 1.0f64 / SELECT_RANGE as f64;
        const EPS: f64 = 0.0001;
        let mut selected_times = [0; SELECT_RANGE];
        let cost = std::time::Instant::now();
        for _ in 0..SELECT_TIMES {
            average_select_indexs(SELECT_RANGE, SELECT_NUM)
                .iter()
                .for_each(|num| selected_times[*num] += 1);
        }
        println!("cost={:?}", cost.elapsed());
        let mut exceed_deviation_num = 0;
        for index in 0..SELECT_RANGE {
            let real_probability = selected_times[index] as f64 / ALL_SELECT_NUM as f64;
            println!(
                "selected_times[{}]={}, theo_probability={}, real_probability={}",
                index, selected_times[index], THEO_PROBABILITY, real_probability
            );
            let deviation = (real_probability - THEO_PROBABILITY).abs();
            if deviation > EPS {
                exceed_deviation_num += 1;
                println!("deviation={} > {}", deviation, EPS);
            }
        }
        println!("exceed_deviation_num={}", exceed_deviation_num);
    }

    #[test]
    fn test_average_select_index_multi_thread() {
        let handles: Vec<_> = (0..5)
            .map(|_| {
                std::thread::spawn(|| {
                    test_average_select_indexs();
                })
            })
            .collect();
        handles
            .into_iter()
            .for_each(|handle| handle.join().unwrap());
    }

    #[test]
    fn test_weight_select_index() {
        const SELECT_TIMES: usize = 10_000_000;
        const SELECT_RANGE: usize = 10;
        const EPS: f64 = 0.0001;
        let mut weights = vec![0.0f64; SELECT_RANGE];
        weights
            .iter_mut()
            .for_each(|weight| *weight = rand::thread_rng().gen_range(0..100) as f64);
        let mut selected_times = [0; SELECT_RANGE];
        normalize(weights.as_mut_slice());
        let cost = std::time::Instant::now();
        for _ in 0..SELECT_TIMES {
            selected_times[weight_select_index(&weights).unwrap()] += 1;
        }
        println!("cost={:?}", cost.elapsed());
        let mut exceed_deviation_num = 0;
        for index in 0..SELECT_RANGE {
            let real_probability = selected_times[index] as f64 / SELECT_TIMES as f64;
            println!(
                "selected_times[{}]={}, theo_probability={}, real_probability={}",
                index, selected_times[index], weights[index], real_probability
            );
            let deviation = (real_probability - weights[index]).abs();
            if deviation > EPS {
                exceed_deviation_num += 1;
                println!("deviation={} > {}", deviation, EPS);
            }
        }
        println!("exceed_deviation_num={}", exceed_deviation_num);
    }

    #[test]
    fn test_weight_select_indexs() {
        const SELECT_TIMES: usize = 10_000_000;
        const SELECT_RANGE: usize = 100;
        const SELECT_NUM: usize = 5;
        const ALL_SELECT_NUM: usize = SELECT_TIMES * SELECT_NUM;
        const EPS: f64 = 0.0001;
        let mut weights = vec![0.0f64; SELECT_RANGE];
        weights
            .iter_mut()
            .for_each(|weight| *weight = rand::thread_rng().gen_range(0..100) as f64);
        let mut selected_times = [0; SELECT_RANGE];
        normalize(weights.as_mut_slice());
        let cost = std::time::Instant::now();
        for _ in 0..SELECT_TIMES {
            weight_select_indexs(&weights, SELECT_NUM)
                .unwrap()
                .iter()
                .for_each(|num| selected_times[*num] += 1)
        }
        println!("cost={:?}", cost.elapsed());
        let mut exceed_deviation_num = 0;
        for index in 0..SELECT_RANGE {
            let real_probability = selected_times[index] as f64 / ALL_SELECT_NUM as f64;
            println!(
                "selected_times[{}]={}, theo_probability={}, real_probability={}",
                index, selected_times[index], weights[index], real_probability
            );
            let deviation = (real_probability - weights[index]).abs();
            if deviation > EPS {
                exceed_deviation_num += 1;
                println!("deviation={} > EPS", deviation);
            }
        }
        println!("exceed_deviation_num={}", exceed_deviation_num);
    }

    // #[test]
    // fn test_euler_angles_to_vector() {
    //     let vector = Vec3::new(1.0, 1.0, 1.0);
    //     let euler_angles = Vec3::new(16.05, 100.2, -78.37);
    //     let result = math::apply_euler_angles_to_vector(&vector, &euler_angles);
    //     println!("result={:?}", result);
    // }
}
