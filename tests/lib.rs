use std::{collections::HashMap, iter::FromIterator};
use thisisplural::Plural;

#[test]
fn example() {
    #[derive(Plural)]
    struct Numbers(Vec<u32>);

    // use From trait
    let mut my_favorite_numbers: Numbers = vec![].into();

    // use DerefMut trait.
    my_favorite_numbers.push(42);

    // HashMap is also supported
    #[derive(Plural)]
    struct FavoriteNumbers(HashMap<&'static str, Numbers>);

    // use FromIterator trait
    let favorite_numbers = FavoriteNumbers::from_iter([("ryo33", my_favorite_numbers)]);

    // use IntoIterator trait
    for (name, numbers) in favorite_numbers {
        // use Deref trait
        println!("{} has {} favorite number(s)", name, numbers.len());
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
fn vec_iter() {
    let tuple = VecTuple(vec![1]);
    let mut iter = tuple.iter();
    let a: Option<&u8> = iter.next();
    assert_eq!(a, Some(&1));
    assert_eq!(iter.next(), None);
}

#[test]
fn vec_iter_mut() {
    let mut tuple = VecTuple(vec![1]);
    let mut iter = tuple.iter_mut();
    let a: Option<&mut u8> = iter.next();
    assert_eq!(a, Some(&mut 1));
    assert_eq!(iter.next(), None);
}

#[test]
fn vec_deref() {
    assert_eq!(VecTuple(vec![1, 2]).len(), 2);
}

#[test]
fn vec_deref_mut() {
    let mut extended = VecTuple(vec![1, 2]);
    extended.extend(VecTuple(vec![3]));
    assert_eq!(extended, VecTuple(vec![1, 2, 3]));
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
fn hash_map_iter() {
    let tuple = HashMapTuple::from_iter([(1, true)]);
    let mut iter = tuple.iter();
    let a: Option<(&u8, &bool)> = iter.next();
    assert_eq!(a, Some((&1, &true)));
    assert_eq!(iter.next(), None);
}

#[test]
fn hash_map_iter_mut() {
    let mut tuple = HashMapTuple::from_iter([(1, true)]);
    let mut iter = tuple.iter_mut();
    let a: Option<(&u8, &mut bool)> = iter.next();
    assert_eq!(a, Some((&1, &mut true)));
    assert_eq!(iter.next(), None);
}

#[test]
fn hash_map_deref() {
    assert_eq!(HashMapTuple::from_iter([(1, true), (2, false)]).len(), 2);
}

#[test]
fn hash_map_deref_mut() {
    let mut extended = HashMapTuple::from_iter([(1, true)]);
    extended.extend(HashMapTuple::from_iter([(2, false)]));
    assert_eq!(extended, HashMapTuple::from_iter([(1, true), (2, false)]));
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