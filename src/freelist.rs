use crate::{Allocator, Policy};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
struct FreeNode {
    addr: usize,
    size: usize,
}

impl FreeNode {
    pub fn new(addr: usize, size: usize) -> FreeNode {
        FreeNode { addr, size }
    }
}

#[derive(Debug, Clone)]
pub struct FreeList {
    max_size: usize,
    align: usize,
    policy: Policy,
    coalesce: bool,
    freelist: Vec<FreeNode>,
    sizemap: HashMap<usize, (usize, usize)>,
}

impl FreeList {
    pub fn new(base_addr: usize, max_size: usize, coalesce: bool) -> FreeList {
        if max_size == 0 {
            panic!("Don't make a heap with size 0");
        }
        let freelist = vec![FreeNode::new(base_addr, max_size)];

        FreeList {
            max_size,
            align: 0,
            policy: Policy::Best,
            coalesce,
            freelist,
            sizemap: HashMap::new(),
        }
    }

    pub fn align(mut self, align: usize) -> Self {
        self.align = align;
        self
    }

    pub fn policy(mut self, policy: Policy) -> Self {
        self.policy = policy;
        self
    }

    // For this to even work well, the freelist
    // needs to be sorted by address
    fn coalesce(&mut self) {
        let mut newlist = Vec::new();
        let mut curr = self.freelist[0];

        for node in self.freelist[1..].iter() {
            if node.addr == curr.addr + curr.size {
                curr = FreeNode::new(curr.addr, curr.size + node.size);
            } else {
                newlist.push(curr);
                curr = *node;
            }
        }

        newlist.push(curr);
        self.freelist = newlist;
    }

    fn best(&self, size: usize) -> Option<usize> {
        let mut bestsize = self.max_size;
        let mut idx: Option<usize> = None;

        for (i, node) in self.freelist.iter().enumerate() {
            if (size <= node.size) && (node.size <= bestsize) {
                idx = Some(i);
                bestsize = node.size;
            }
        }

        idx
    }

    fn first(&self, size: usize) -> Option<usize> {
        for (i, node) in self.freelist.iter().enumerate() {
            if size <= node.size {
                return Some(i);
            }
        }
        None
    }

    fn check_size(&self, mut size: usize) -> bool {
        if self.align > 1 {
            let left = size % self.align;
            if left != 0 {
                size += self.align - left;
            }
        }

        let idx = match self.policy {
            Policy::Best => self.best(size),
            Policy::First => self.first(size),
        };

        idx.is_some()
    }
}

impl Allocator for FreeList {
    fn malloc(&mut self, mut size: usize) -> Option<usize> {
        let mut diff = 0;
        if self.align > 1 {
            let left = size % self.align;
            if left != 0 {
                diff = self.align - left;
                size += diff;
            }
        }

        let idx = match self.policy {
            Policy::Best => self.best(size),
            Policy::First => self.first(size),
        };

        if let Some(i) = idx {
            let node = self.freelist[i];
            self.sizemap.insert(node.addr, (size, diff));
            match size.cmp(&node.size) {
                Ordering::Equal => {
                    self.freelist.remove(i);
                }
                Ordering::Less => {
                    self.freelist[i] = FreeNode::new(node.addr + size, node.size - size);
                }
                Ordering::Greater => panic!("Not possible"),
            }

            return Some(node.addr);
        }

        None
    }

    fn free(&mut self, ptr: usize) -> Result<(), &str> {
        // Get the size from the sizemap, remove it
        // from map if exist else, return err
        let (size, _) = self.sizemap.remove(&ptr).ok_or("Pointer not found")?;

        // insert back
        self.freelist.push(FreeNode::new(ptr, size));
        self.freelist.sort_unstable_by_key(|node| node.addr);

        // Coalesce if the flag is set
        if self.coalesce {
            self.coalesce()
        }

        Ok(())
    }

    fn largest_alloc(&self) -> usize {
        (1..=self.max_size + 1)
            .into_iter()
            .find(|&x| !self.check_size(x))
            .unwrap()
            - 1
    }

    fn free_space(&self) -> usize {
        self.freelist.iter().map(|node| node.size).sum()
    }

    fn internal_frag(&self) -> usize {
        self.sizemap.iter().map(|(&_, &(_, diff))| diff).sum()
    }

    fn print(&self) {
        let len = self.freelist.len();

        for i in 0..len {
            print!("\u{250c}{:\u{2500}<12}\u{2510}", "");
            if i == len - 1 {
                println!();
            } else {
                print!("{:<5}", "");
            }
        }
        for i in 0..len {
            print!("\u{2502} addr: {:<4} \u{2502}", self.freelist[i].addr);
            if i == len - 1 {
                println!();
            } else {
                print!(" --\\ ");
            }
        }
        for i in 0..len {
            print!("\u{2502} size: {:<4} \u{2502}", self.freelist[i].size);
            if i == len - 1 {
                println!();
            } else {
                print!(" --/ ");
            }
        }
        for i in 0..len {
            print!("\u{2514}{:\u{2500}<12}\u{2518}", "");
            if i == len - 1 {
                println!();
            } else {
                print!("{:<5}", "");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malloc() {
        let mut list = FreeList::new(1000, 100, false);
        list.malloc(4);
        let node = list.freelist.pop().unwrap();
        assert_eq!(node.addr, 1004);
        assert_eq!(node.size, 96);
    }

    #[test]
    fn free() {
        let mut list = FreeList::new(1000, 100, false);
        // Illegal free
        assert!(list.free(0).is_err());

        let addr = list.malloc(10).unwrap();
        assert!(list.free(addr).is_ok());
        // Double free
        assert!(list.free(addr).is_err());
    }

    #[test]
    fn free_space() {
        let max_space = 100;
        let mut list = FreeList::new(1000, max_space, false);
        assert_eq!(list.free_space(), max_space);
        list.malloc(10);
        assert_eq!(list.free_space(), max_space - 10);
    }

    #[test]
    fn internal_fragmentation() {
        let mut list = FreeList::new(1000, 100, false).align(4);
        let mut ptrs = Vec::new();

        ptrs.push(list.malloc(7).unwrap());
        assert_eq!(list.internal_frag(), 1);
        ptrs.push(list.malloc(7).unwrap());
        assert_eq!(list.internal_frag(), 2);
        list.free(ptrs.pop().unwrap()).unwrap();
        assert_eq!(list.internal_frag(), 1);
    }

    #[test]
    fn largest_alloc() {
        let mut list = FreeList::new(1000, 100, false).align(4);
        assert_eq!(list.largest_alloc(), 100);
        list.malloc(16);
        assert_eq!(list.largest_alloc(), 84);
        list.malloc(1);
        assert_eq!(list.largest_alloc(), 80);
    }
}
