use craft_crypto::emoji::EmojiScript;
use soul_bus::{BusError, SoulBus};

#[test]
fn two_souls_independent() {
    let bus = SoulBus::new();
    bus.spawn_soul(1, None).unwrap();
    bus.spawn_soul(2, None).unwrap();

    let first = bus.load(1, &EmojiScript, "🔢6 🔢7 ✖️ ↩️").unwrap();
    let second = bus.load(2, &EmojiScript, "🔢100 🔢1 ➖ ↩️").unwrap();

    assert_eq!(bus.call(1, &first, vec![]), Ok(42));
    assert_eq!(bus.call(2, &second, vec![]), Ok(99));
}

#[test]
fn broadcast() {
    let bus = SoulBus::new();
    let mut name = String::new();

    for soul_id in [10, 11, 12] {
        bus.spawn_soul(soul_id, None).unwrap();
        name = bus.load(soul_id, &EmojiScript, "🔢1 ↩️").unwrap();
    }

    let results = bus.broadcast(&name, vec![]);
    assert_eq!(results.len(), 3);
    assert!(results.into_iter().all(|(_, result)| result == Ok(1)));
}

#[test]
fn unknown_soul_error() {
    let bus = SoulBus::new();
    assert_eq!(
        bus.call(99, "anything", vec![]),
        Err(BusError::UnknownSoul(99))
    );
}

#[test]
fn despawn_removes_soul() {
    let bus = SoulBus::new();
    bus.spawn_soul(5, None).unwrap();
    bus.despawn(5).unwrap();

    assert_eq!(bus.call(5, "x", vec![]), Err(BusError::UnknownSoul(5)));
    assert!(!bus.soul_alive(5));
}
