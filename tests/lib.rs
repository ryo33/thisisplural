use std::{collections::HashMap, iter::FromIterator};
use thisisplural::Plural;

#[test]
fn example() {
    // This implements `From`, `Into`, `FromIterator`, `IntoIterator`, `Deref`, and `DerefMut`.
    #[derive(Plural)]
    struct Numbers(Vec<u32>);

    // use `From` trait
    let my_favorite_numbers: Numbers = vec![].into();

    // `FromIterator` allows this `collect()`
    let doubled_numbers: Numbers = my_favorite_numbers.0.iter().map(|x| x * 2).collect();

    // `HashMap` is also supported
    #[derive(Plural)]
    struct FavoriteNumbers(HashMap<&'static str, Numbers>);

    // construct the struct with using `FromIterator`
    let favorite_numbers =
        FavoriteNumbers::from_iter([("ryo33", my_favorite_numbers), ("someone", doubled_numbers)]);

    // use it in a `for` loop (`IntoIterator` trait)
    for (name, numbers) in favorite_numbers {
        // use Deref trait
        println!("{} has {} favorite number(s)", name, numbers.0.len());
    }
}

#[derive(Plural, Debug, PartialEq)]
struct VecTuple(Vec<u8>);

#[test]
fn vec_into_iter() {
    assert_eq!(
        IntoIterator::into_iter(VecTuple(vec![1, 2])).collect::<Vec<_>>(),
        vec![1, 2]
    );
}

#[test]
fn vec_collect() {
    assert_eq!(std::iter::once(1).collect::<VecTuple>(), VecTuple(vec![1]));
}

#[test]
fn vec_into() {
    let vec: Vec<_> = VecTuple(vec![1]).into();
    assert_eq!(vec, vec![1]);
}

#[test]
fn vec_from() {
    assert_eq!(VecTuple::from(vec![1]), VecTuple(vec![1]));
}

#[test]
fn supports_pub() {
    #[derive(Plural, Debug, PartialEq)]
    struct VecTuple(pub(crate) Vec<u8>);
}

#[test]
fn vec_supports_trait_bound() {
    #[derive(Plural, Debug, PartialEq)]
    struct VecTuple<'a, T, const N: usize>(Vec<[&'a T; N]>);
}

#[derive(Plural, Debug, PartialEq)]
struct HashMapTuple(std::collections::HashMap<u8, bool>);

#[test]
fn hash_map_into_iter() {
    assert_eq!(
        IntoIterator::into_iter(HashMapTuple::from_iter([(1, true), (2, false)]))
            .collect::<HashMap<_, _>>(),
        HashMap::from_iter([(1, true), (2, false)])
    );
}

#[test]
fn hash_map_collect() {
    assert_eq!(
        std::iter::once((1, true)).collect::<HashMapTuple>(),
        HashMapTuple(HashMap::from_iter([(1, true)]))
    );
}
#[test]
fn hash_map_into() {
    let hash_map: HashMap<_, _> = HashMapTuple::from_iter([(1, true)]).into();
    assert_eq!(hash_map, HashMap::from_iter([(1, true)]));
}

#[test]
fn hash_map_from() {
    assert_eq!(
        HashMapTuple::from(HashMap::from([(1, true)])),
        HashMapTuple(HashMap::from([(1, true)]))
    );
}

#[test]
fn hash_map_supports_trait_bounds() {
    #[derive(Plural)]
    struct VecTuple<K: Eq + std::hash::Hash, V>(HashMap<K, V>);
}
