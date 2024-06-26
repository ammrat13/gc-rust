pub mod allocations;
pub mod refs;

mod arch;

#[cfg(test)]
mod test;

use std::fs::File;
use std::io::{BufRead, Result};

use crate::allocations::GCAllocation;
use crate::refs::GCRef;

pub struct GCContext {
    allocations: Vec<GCAllocation>,
}

impl GCContext {
    pub fn new() -> Self {
        GCContext {
            allocations: Vec::new(),
        }
    }

    pub fn allocations(&self) -> &[GCAllocation] {
        &self.allocations
    }

    pub fn allocate<T>(&mut self, value: T) -> GCRef<T> {
        // Allocate the value, and keep the layout. Remember to write the actual
        // data into the location.
        //
        // SAFETY: We correctly initialize the data before using it.
        let layout = std::alloc::Layout::new::<T>();
        let ptr = unsafe { std::alloc::alloc(layout) } as *mut T;
        unsafe { std::ptr::write(ptr, value) };
        // Remember the allocation
        self.allocations.push(GCAllocation {
            layout,
            marked: false,
            start: ptr as usize,
        });
        // Return
        GCRef { ptr }
    }

    pub fn collect(&mut self) -> Result<()> {
        // Get the current stack pointer. Make sure to clobber all the other
        // caller-save registers to make sure they make it onto the stack. We
        // don't need to account for the redzone since we call another function.
        let stack_pointer = crate::arch::get_stack_pointer();
        // Call the inner function. This way, we know everything above us won't
        // be counted.
        self.collect_inner(stack_pointer)
    }

    #[inline(never)]
    fn collect_inner(&mut self, stack_pointer: usize) -> Result<()> {
        // Initially, all the allocations are unmarked
        for allocation in self.allocations.iter_mut() {
            allocation.marked = false;
        }

        // Open the file with all our mappings. We will iterate over these
        // mappings, marking them.
        let maps = File::open("/proc/self/maps")?;
        let maps_reader = std::io::BufReader::new(maps);
        for map_result in maps_reader.lines() {
            self.mark_map(&map_result?, stack_pointer);
        }

        // Sweep unmarked allocations. Make sure to actually free the data.
        for allocation in self.allocations.iter() {
            if !allocation.marked {
                // SAFETY: We know that the allocation is not marked, so it's
                // safe to free it. Allocations on the heap can't count on
                // getting their drop function called, so we can just not do it.
                unsafe {
                    std::alloc::dealloc(allocation.start as *mut u8, allocation.layout);
                }
            }
        }
        self.allocations.retain(|a| a.marked);

        // Done
        Ok(())
    }

    fn mark_map(&mut self, map: &str, stack_pointer: usize) {
        println!("{}", map);
        // Parse the mapping. We just need to know the memory addresses and
        // whether this is the stack. The file has a stable format, so if we
        // don't get that just panic.
        let (map_start, map_end, writable, is_stack) = {
            // Do the initial string parsing
            let mut parts = map.split_whitespace();
            let range = parts.nth(0).unwrap();
            let perms = parts.nth(0).unwrap();
            // Parse the range
            let mut range_parts = range.split('-');
            let start = usize::from_str_radix(range_parts.nth(0).unwrap(), 16).unwrap();
            let end = usize::from_str_radix(range_parts.nth(0).unwrap(), 16).unwrap();
            // Parse the permissions. We just need to check if this region is
            // writable and not executable - i.e. it contains data.
            let writable = perms.starts_with("rw-");
            // Check whether this mapping is for the stack. There's a
            // pseudo-path for this, so check that. Additionally, check if the
            // current stack pointer is within this mapping. This allows tests
            // to pass since they run on a different stack.
            let is_stack = map.ends_with("[stack]") || (start <= stack_pointer && stack_pointer < end);
            // Return
            (start, end, writable, is_stack)
        };

        // If this mapping is not writable, there's no way for it to contain a
        // pointer to the heap.
        if !writable {
            return;
        }

        // Mark the entire region. If this region is the stack, we only need to
        // check past the caller's stack pointer.
        let mark_start = if is_stack {
            std::cmp::max(stack_pointer, map_start)
        } else {
            map_start
        };
        self.mark_range(mark_start, map_end, true)
    }

    fn mark_range(&mut self, start: usize, end: usize, first: bool) {
        // TODO: Optimize this function. Currently, it loops over every
        // allocation for every address, which is very slow. Ideally, we'd work
        // in our own contiguous arena to make the checks much faster. But, this
        // is good enough for a demo.

        // If we don't have anything to mark, just return.
        if start >= end {
            return;
        }
        // Make sure both parameters are aligned to the size of a pointer.
        let psize = crate::arch::get_pointer_size();
        let start = ((start + psize - 1) / psize) * psize;
        let end = (end / psize) * psize;

        // Iterate over all pointers in the range. Note that the end is
        // exclusive.
        for cur in (start..end).step_by(8) {
            // Convert to a pointer and read the value
            let value = unsafe { *(cur as *const usize) };

            // Ignore pointers from the metadata vector itself.
            let metadata_range = self.allocations.as_ptr_range();
            if cur >= metadata_range.start as usize && cur < metadata_range.end as usize {
                continue;
            }
            // If this is our first call, ignore pointers from within the
            // allocated chunks.
            if first {
                let cur_allocation_opt = self.allocations.iter().find(|a| a.contains(cur));
                if cur_allocation_opt.is_some() {
                    continue;
                }
            }

            // Check to see if the memory location points to any allocation. If
            // it points to no allocation, continue.
            let allocation_opt = self.allocations.iter_mut().find(|a| a.contains(value));
            if allocation_opt.is_none() {
                continue;
            }
            // If it does point to an allocation, mark it. Remember whether it
            // was marked before.
            let allocation = allocation_opt.unwrap();
            let was_marked = allocation.marked;
            allocation.marked = true;

            println!(
                "Found {:p} -> {:p}",
                cur as *const usize, value as *const usize
            );

            // Finally, mark recursively if the allocation wasn't already
            // marked. Note that we have to copy the start and end.
            let (s, e) = (allocation.start, allocation.end());
            if !was_marked {
                self.mark_range(s, e, false);
            }
        }
    }
}
