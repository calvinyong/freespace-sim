use crate::buddy::BuddyAllocator;
use crate::freelist::FreeList;
use crate::{Allocator, Policy};

pub fn freelist(coalesce: bool) {
    println!(
        "Demoing freelist{}\n",
        if coalesce { " with coalescing" } else { "" }
    );
    let mut list = FreeList::new(0, 1024, coalesce)
        .align(4)
        .policy(Policy::Best);

    let mut ptr;
    let mut ptrs = Vec::new();
    let calls: [isize; 5] = [7, 0, 9, -1, 12];

    for &i in calls.iter() {
        if i > 0 {
            ptr = list.malloc(i as usize).unwrap();
            println!("malloc({}) returned {}", i, ptr);
            ptrs.push(ptr);
        } else {
            ptr = ptrs[i.abs() as usize];
            println!("Freeing ptr {}", ptr);
            list.free(ptr).expect("Free failed");
        }

        list.print();
    }

    println!("Internal fragmentation: {}", list.internal_frag());
    println!("External fragmentation: {}", list.external_frag());
}

pub fn buddy() {
    println!("Demoing buddy allocator\n");
    let mut buddy = BuddyAllocator::new(0, 3);

    println!("Initial buddy allocator, min size 1, max size 8");
    buddy.print();
    println!();

    for _ in 0..3 {
        let ptr = buddy.malloc(1).unwrap();
        println!("malloc(1) returned {}", ptr);
        buddy.print();
        println!();
    }

    println!("Internal fragmentation: {}", buddy.internal_frag());
    println!("External fragmentation: {}\n", buddy.external_frag());

    buddy.free(2).unwrap();

    println!("Buddy after freeing ptr 2");
    buddy.print();
    println!("Internal fragmentation: {}", buddy.internal_frag());
    println!("External fragmentation: {}", buddy.external_frag());
}
