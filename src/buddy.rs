use super::Allocator;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
struct Block {
    addr: usize,
    size_class: usize,
    //available: bool,
}

impl Block {
    pub fn new(addr: usize, size_class: usize) -> Self {
        Self { addr, size_class }
    }

    pub fn buddy(&self) -> Self {
        Self {
            addr: self.addr ^ (1 << self.size_class),
            size_class: self.size_class,
        }
    }
}

#[derive(Debug, Clone)]
struct Level {
    blocks: Vec<Block>,
    size_class: usize,
}

impl Level {
    fn new(size_class: usize) -> Self {
        Self {
            blocks: Vec::new(),
            size_class,
        }
    }

    fn has_available_block(&self) -> bool {
        !self.blocks.is_empty()
    }

    fn add(&mut self, block: Block) {
        self.blocks.push(block);
    }

    fn pop_front(&mut self) -> Block {
        self.blocks.remove(0)
    }
}

#[derive(Debug, Clone)]
pub struct BuddyAllocator {
    min_size: usize,
    max_size: usize,
    levels: Vec<Level>,
    sizemap: HashMap<usize, (usize, usize)>,
}

impl BuddyAllocator {
    pub fn new(min_size: usize, max_size: usize) -> Self {
        let mut levels = Vec::with_capacity(max_size - min_size + 1);

        for i in min_size..max_size {
            levels.push(Level::new(i));
        }

        let mut last_level = Level::new(max_size);
        last_level.add(Block::new(0, max_size));
        levels.push(last_level);

        Self {
            min_size,
            max_size,
            levels,
            sizemap: HashMap::new(),
        }
    }

    fn size_class_to_index(&self, size_class: usize) -> usize {
        size_class - self.min_size
    }

    fn check_size(&self, size: usize) -> bool {
        let mut j = (size as f32).log2().ceil() as usize;
        if j > self.max_size {
            return false;
        } else if j < self.min_size {
            j = self.min_size;
        }

        let idx = self.size_class_to_index(j);
        if self.levels[idx].has_available_block() {
            return true;
        }

        let mut curr_size_class = j + 1;

        // Go up the levels to find free space
        while curr_size_class <= self.max_size {
            let idx = self.size_class_to_index(curr_size_class);
            if self.levels[idx].has_available_block() {
                break;
            }
            curr_size_class += 1;
        }

        curr_size_class != self.max_size + 1
    }
}

impl Allocator for BuddyAllocator {
    fn malloc(&mut self, size: usize) -> Option<usize> {
        // Smallest power that can accommodate the requested size
        let mut j = (size as f32).log2().ceil() as usize;
        // Too big
        if j > self.max_size {
            return None;
        } else if j < self.min_size {
            // clamp lower bound
            j = self.min_size;
        }

        let diff = (1 << j) - size;

        // Check if current level has free space
        let idx = self.size_class_to_index(j);
        if self.levels[idx].has_available_block() {
            let block = self.levels[idx].pop_front();
            self.sizemap.insert(block.addr, (j, diff));
            return Some(block.addr);
        }

        let mut idx = 0;
        let mut curr_size_class = j + 1;

        // Go up the levels to find free space
        while curr_size_class <= self.max_size {
            idx = self.size_class_to_index(curr_size_class);
            if self.levels[idx].has_available_block() {
                break;
            }
            curr_size_class += 1;
        }

        // No free space
        if curr_size_class == self.max_size + 1 {
            return None;
        }

        let mut block = self.levels[idx].pop_front();
        curr_size_class -= 1;
        while curr_size_class >= j {
            idx = self.size_class_to_index(curr_size_class);
            let block1 = Block::new(block.addr, curr_size_class);
            let buddy = block1.buddy();

            block = block1;
            self.levels[idx].add(buddy);

            if curr_size_class == 0 {
                break;
            } else {
                curr_size_class -= 1;
            }
        }

        self.sizemap.insert(block.addr, (j, diff));
        Some(block.addr)
    }

    fn free(&mut self, ptr: usize) -> Result<(), &str> {
        let (mut size_class, _) = self.sizemap.remove(&ptr).ok_or("pointer not found")?;

        while size_class <= self.max_size {
            let i = self.size_class_to_index(size_class);
            let block = Block::new(ptr, size_class);
            let buddy = self.levels[i]
                .blocks
                .iter()
                .position(|b| b.addr == block.buddy().addr);

            // If found buddy in free list, then we can coalesce
            if let Some(buddy_index) = buddy {
                self.levels[i].blocks.remove(buddy_index);
            } else {
                self.levels[i].add(block);
                break;
            }
            size_class += 1;
        }

        Ok(())
    }

    fn largest_alloc(&self) -> usize {
        (1..=(1 << self.max_size) + 1)
            .into_iter()
            .find(|&x| !self.check_size(x))
            .unwrap()
            - 1
    }

    fn free_space(&self) -> usize {
        self.levels
            .iter()
            .map(|level| (1 << level.size_class) * level.blocks.len())
            .sum()
    }

    fn internal_frag(&self) -> usize {
        self.sizemap.iter().map(|(&_, &(_, diff))| diff).sum()
    }

    fn print(&self) {
        self.levels
            .iter()
            .rev()
            .for_each(|level| println!("Size class {}: {:?}", level.size_class, level.blocks));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn malloc() {
        let mut buddy = BuddyAllocator::new(0, 2);

        assert_eq!(buddy.malloc(1).unwrap(), 0);
        assert_eq!(buddy.malloc(2).unwrap(), 2);
        assert_eq!(buddy.malloc(1).unwrap(), 1);
        assert!(buddy.malloc(1).is_none());
    }

    #[test]
    fn malloc_too_big() {
        let mut buddy = BuddyAllocator::new(2, 5);
        assert!(buddy.malloc(64).is_none());
    }

    #[test]
    fn free() {
        let mut buddy = BuddyAllocator::new(0, 3);
        println!("Init");
        buddy.print();

        let ptr = buddy.malloc(1).unwrap();
        println!("After malloc 1");
        buddy.print();
        assert!(buddy.free(ptr).is_ok());

        println!("After free");
        buddy.print();
        assert_eq!(buddy.free_space(), 8);

        buddy.malloc(1);
        let ptr = buddy.malloc(1).unwrap();
        buddy.print();
        assert!(buddy.free(ptr).is_ok());
        assert_eq!(buddy.free_space(), 7);
    }

    #[test]
    fn illegal_free() {
        let mut buddy = BuddyAllocator::new(0, 3);
        let ptr = 4;
        assert!(buddy.free(ptr).is_err());
        buddy.malloc(4);
        assert!(buddy.free(ptr).is_err());
    }

    #[test]
    fn double_free() {
        let mut buddy = BuddyAllocator::new(0, 3);
        let ptr = buddy.malloc(2).unwrap();
        assert!(buddy.free(ptr).is_ok());
        assert!(buddy.free(ptr).is_err());
    }

    #[test]
    fn free_hard() {
        let mut buddy = BuddyAllocator::new(1, 3);
        let ptr1 = buddy.malloc(1).unwrap();
        assert!(buddy.free(ptr1).is_ok());

        let mut buddy = BuddyAllocator::new(1, 3);
        let ptr1 = buddy.malloc(1).unwrap();
        let ptr2 = buddy.malloc(1).unwrap();
        assert!(buddy.free(ptr2).is_ok());
        assert!(buddy.free(ptr1).is_ok());
    }

    #[test]
    fn internal_fragmentation() {
        let mut buddy = BuddyAllocator::new(1, 3);
        for _ in 0..4 {
            buddy.malloc(2);
        }
        assert_eq!(buddy.internal_frag(), 0);

        let mut buddy = BuddyAllocator::new(1, 3);
        for _ in 0..4 {
            buddy.malloc(1);
        }
        assert_eq!(buddy.internal_frag(), 4);
    }

    #[test]
    fn largest_alloc() {
        let mut buddy = BuddyAllocator::new(1, 3);
        assert_eq!(buddy.largest_alloc(), 8);
        buddy.malloc(2);
        assert_eq!(buddy.largest_alloc(), 4);
        buddy.malloc(2);
        assert_eq!(buddy.largest_alloc(), 4);
    }

    #[test]
    fn extreme_ext_frag() {
        // Extreme case
        let mut buddy = BuddyAllocator::new(0, 3);
        for _ in 0..8 {
            buddy.malloc(1);
        }
        for i in (0..8).step_by(2) {
            assert!(buddy.free(i).is_ok());
        }

        assert_eq!(buddy.largest_alloc(), 1);
    }

    #[test]
    fn free_space() {
        let mut buddy = BuddyAllocator::new(1, 3);
        assert_eq!(buddy.free_space(), 8);
        buddy.malloc(2);
        assert_eq!(buddy.free_space(), 6);
    }

    #[test]
    fn size_class_match() {
        let mut buddy = BuddyAllocator::new(0, 3);
        for _ in 0..8 {
            buddy.malloc(1);
        }
        for i in (0..8).step_by(2) {
            assert!(buddy.free(i).is_ok());
        }

        buddy.print();

        buddy.free(1).unwrap();
        buddy.free(3).unwrap();
        buddy.free(5).unwrap();

        for (i, level) in buddy.levels.iter().enumerate() {
            for block in level.blocks.iter() {
                assert_eq!(block.size_class, i + buddy.min_size);
            }
        }
    }
}
