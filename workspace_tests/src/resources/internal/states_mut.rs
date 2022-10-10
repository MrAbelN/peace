use peace::{
    cfg::{item_spec_id, ItemSpecId},
    resources::{internal::StatesMut, states::ts::Current, type_reg::untagged::TypeMap},
};

#[test]
fn with_capacity_reserves_enough_capacity() {
    let states = StatesMut::<Current>::with_capacity(100);
    assert!(states.capacity() >= 100);
}

#[test]
fn into_inner() {
    let states = test_states();

    let type_map = states.into_inner();

    assert_eq!(1, type_map.len())
}

#[test]
fn deref_and_deref_mut() {
    let mut states = StatesMut::<Current>::new();

    // deref_mut
    states.insert(item_spec_id!("key"), 123);

    // deref
    assert_eq!(1, states.len())
}

#[test]
fn from_type_map() {
    let _states = StatesMut::<Current>::from(TypeMap::new_typed());
}

#[test]
fn debug() {
    let states = test_states();

    assert_eq!(
        r#"StatesMut({ItemSpecId("key"): TypedValue { type: "i32", value: 123 }}, PhantomData<peace_resources::states::ts::Current>)"#,
        format!("{states:?}")
    );
}

fn test_states() -> StatesMut<Current> {
    let mut states = StatesMut::<Current>::new();
    states.insert(item_spec_id!("key"), 123);

    states
}