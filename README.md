# `rod_map`, remove on drop maps

**This is experimental work**

`rod_map` offers two type of maps:
- `RodHashMap`
- `RodBTreeMap`

These maps return a handle when a value is inserted and will automatically remove that value when all handles are dropped.

## Example
```rust
type RoomId = usize;
struct Room;

let mut hotel = RodHashMap::<RoomId, Room>::new();

assert!(hotel.is_empty());

// Let's get a room.
let key = hotel.insert(0, Room);

// Keep a spare key.
let spare_key = key.clone();

// The key is our handle to the room, it can be cloned
// or retrieved from the hotel with the get method:
assert_eq!(hotel.get(0), Some(key));

// There is one room (ours) in the hotel.
assert_eq!(hotel.len(), 1);

// Dropping our key won't remove the room from the hotel
// because we have a spare.
drop(key);
assert_eq!(hotel.len(), 1);

// Dropping the spare WILL remove the room from the hotel.
drop(spare_key);
assert!(hotel.is_empty());
```

## License

This project is licensed under the [MIT license](LICENSE).