# Free space management simulator

Operating systems project.

This project implements a [buddy allocator](https://en.wikipedia.org/wiki/Buddy_memory_allocation) and explores its efficiency of it in terms of internal and external fragmentation. To benchmark the memory allocator, we implement OSTEP's [`malloc.py`](https://github.com/remzi-arpacidusseau/ostep-homework/tree/master/vm-freespace) in Rust. Since we implemented a list-based freelist and a buddy allocator, this project is called `freespace-sim`.

## Implementation details

* Normally these allocators would be written with doubly linked list, but the purpose of this project is just to simulate them, and not to implement them as if we were writting our own allocator.
* Both allocators do not take into account the size needed for metadata and headers. We just assume this size when requesting memory in our simulations.

## Running the code

Follow the directions at the [Rust website](https://www.rust-lang.org/tools/install) to get Rust.

## Demo the allocators

### List-based Freelist

`cargo run -- demo freelist`

```
Demoing freelist

malloc(7) returned 0
┌────────────┐
│ addr: 8    │
│ size: 1016 │
└────────────┘
Freeing ptr 0
┌────────────┐     ┌────────────┐
│ addr: 0    │ --\ │ addr: 8    │
│ size: 8    │ --/ │ size: 1016 │
└────────────┘     └────────────┘
malloc(9) returned 8
┌────────────┐     ┌────────────┐
│ addr: 0    │ --\ │ addr: 20   │
│ size: 8    │ --/ │ size: 1004 │
└────────────┘     └────────────┘
Freeing ptr 8
┌────────────┐     ┌────────────┐     ┌────────────┐
│ addr: 0    │ --\ │ addr: 8    │ --\ │ addr: 20   │
│ size: 8    │ --/ │ size: 12   │ --/ │ size: 1004 │
└────────────┘     └────────────┘     └────────────┘
malloc(12) returned 8
┌────────────┐     ┌────────────┐
│ addr: 0    │ --\ │ addr: 20   │
│ size: 8    │ --/ │ size: 1004 │
└────────────┘     └────────────┘
Internal fragmentation: 0
External fragmentation: 0.007905126
```

`cargo run -- demo freelist --coalesce`

```
Demoing freelist with coalescing

malloc(7) returned 0
┌────────────┐
│ addr: 8    │
│ size: 1016 │
└────────────┘
Freeing ptr 0
┌────────────┐
│ addr: 0    │
│ size: 1024 │
└────────────┘
malloc(9) returned 0
┌────────────┐
│ addr: 12   │
│ size: 1012 │
└────────────┘
Freeing ptr 0
┌────────────┐
│ addr: 0    │
│ size: 1024 │
└────────────┘
malloc(12) returned 0
┌────────────┐
│ addr: 12   │
│ size: 1012 │
└────────────┘
Internal fragmentation: 0
External fragmentation: 0
```

### Buddy allocator

`cargo run -- demo buddy`

```
Demoing buddy allocator

Initial buddy allocator, min size 1, max size 8
Size class 3: [Block { addr: 0, size_class: 3 }]
Size class 2: []
Size class 1: []
Size class 0: []

malloc(1) returned 0
Size class 3: []
Size class 2: [Block { addr: 4, size_class: 2 }]
Size class 1: [Block { addr: 2, size_class: 1 }]
Size class 0: [Block { addr: 1, size_class: 0 }]

malloc(1) returned 1
Size class 3: []
Size class 2: [Block { addr: 4, size_class: 2 }]
Size class 1: [Block { addr: 2, size_class: 1 }]
Size class 0: []

malloc(1) returned 2
Size class 3: []
Size class 2: [Block { addr: 4, size_class: 2 }]
Size class 1: []
Size class 0: [Block { addr: 3, size_class: 0 }]

Internal fragmentation: 0
External fragmentation: 0.19999999

Buddy after freeing ptr 2
Size class 3: []
Size class 2: [Block { addr: 4, size_class: 2 }]
Size class 1: [Block { addr: 2, size_class: 1 }]
Size class 0: []
Internal fragmentation: 0
External fragmentation: 0.3333333
```

## Run the benchmarks

Specify a malloc ratio with `-r` option. Defaults to 0.5.

### Constant size
`cargo run -- bench stack -r 0.5`

```
Fixed size allocation with 50% malloc

Free list results
Average malloc fails: 0
Average free fails: 0
Average internal fragmentation: 0
Average external fragmentation: 0

Buddy allocator results
Average malloc fails: 0
Average free fails: 0
Average internal fragmentation: 0
Average external fragmentation: 0.48845467
```

### Random size


```
Random size allocation with 50% malloc

Free list results
Average malloc fails: 0
Average free fails: 0
Average internal fragmentation: 521.4
Average external fragmentation: 0.0138943195

Buddy allocator results
Average malloc fails: 0
Average free fails: 0
Average internal fragmentation: 727.6
Average external fragmentation: 0.44816023
```
