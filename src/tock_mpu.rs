flux_rs::defs! {
    fn po2(n: int) -> bool {
        bv_int_to_bv32(n) & (bv_int_to_bv32(n) - 1) == 0
    }

    fn kernel_break(start: int, mem_size: int, kernel_size: int) -> int {
        start + mem_size - kernel_size
    }

    fn num_enabled_subregions(app_mem_size: int, region_size: int) -> int {
        ((app_mem_size * 8) / region_size + 1)
    }

    fn subregion_size(region_size: int) -> int {
        region_size / 8
    }

    fn subregion_enabled_end(start: int, app_mem_size: int, region_size: int) -> int {
        start + num_enabled_subregions(app_mem_size, region_size) * subregion_size(region_size)
    }
}


#[flux_rs::sig(fn (num: usize) -> usize {n: n >= num && po2(n) && n / 2 <= num })]
#[flux_rs::trusted]
pub fn closest_power_of_two(mut num: usize) -> usize {
    num -= 1;
    num |= num >> 1;
    num |= num >> 2;
    num |= num >> 4;
    num |= num >> 8;
    num |= num >> 16;
    num += 1;
    num
}

#[flux_rs::sig(fn (app_mem_size: usize, region_size: usize) -> usize[num_enabled_subregions(app_mem_size, region_size)] requires region_size >= 256)]
fn num_enabled_subregions(initial_app_memory_size: usize, region_size: usize) -> usize {
    (initial_app_memory_size * 8) / region_size + 1
}

#[flux_rs::sig(fn (region_size: usize) -> usize[subregion_size(region_size)] requires region_size >= 256)]
fn subregion_size(region_size: usize) -> usize {
    region_size / 8
}

#[flux_rs::sig(fn (start: usize, app_mem_size: usize, region_size: usize) -> usize[subregion_enabled_end(start, app_mem_size, region_size)] requires region_size >= 256)]
fn subregion_enabled_end(start: usize, initial_app_memory_size: usize, region_size: usize) -> usize {
    start + num_enabled_subregions(initial_app_memory_size, region_size) * subregion_size(region_size)
}

#[flux_rs::sig(fn (start: usize, memory_size: usize, initial_kernel_memory_size: usize) -> usize[kernel_break(start, memory_size, initial_kernel_memory_size)] requires initial_kernel_memory_size <= start + memory_size)]
fn kernel_break(start: usize, memory_size: usize, initial_kernel_memory_size: usize) -> usize {
    start + memory_size - initial_kernel_memory_size
}

#[flux_rs::refined_by(subregion_end: int, memory_size: int)]
struct Pair {
    #[field(usize[subregion_end])]
    subregion_end: usize,
    #[field(usize[memory_size])]
    memory_size: usize
}

#[flux_rs::sig(
    fn (
        memory_size_po2: usize,
        region_size: usize,
        region_start: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
    ) -> Pair { new_end: kernel_break(region_start, new_end.memory_size, initial_kernel_memory_size) >= new_end.subregion_end }
        // && po2(new_end.memory_size)}
    requires 
        initial_app_memory_size > 0 // app mem size > 0
        &&
        initial_kernel_memory_size > 0 // kernel mem size > 0
        &&
        initial_app_memory_size + initial_kernel_memory_size <= memory_size_po2
        &&
        memory_size_po2 >= 512 
        // &&
        // po2(memory_size_po2)
        &&
        region_size == memory_size_po2 / 2 
        &&
        subregion_enabled_end(region_start, initial_app_memory_size, region_size) > kernel_break(region_start, memory_size_po2, initial_kernel_memory_size)
)]
fn overlap_body(
    mut memory_size_po2: usize,
    region_size: usize,
    mut region_start: usize,
    initial_app_memory_size: usize,
    initial_kernel_memory_size: usize,
) -> Pair {
    let enabled_end = subregion_enabled_end(region_start, initial_app_memory_size, region_size);
    let kbreak = kernel_break(region_start, memory_size_po2, initial_kernel_memory_size);
    let overlap = enabled_end - kbreak;
    memory_size_po2 = closest_power_of_two(overlap + memory_size_po2);
    let region_size = memory_size_po2 / 2;
    if region_start % region_size != 0 {
        region_start += region_size - (region_start % region_size);
    }
    let x = true;

    Pair { subregion_end: subregion_enabled_end(region_start, initial_app_memory_size, region_size), memory_size: memory_size_po2 }
}