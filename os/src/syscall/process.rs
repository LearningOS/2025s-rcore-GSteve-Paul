//! Process management syscalls
use core::mem::size_of;
use core::slice::from_raw_parts;

use crate::{
    config::PAGE_SIZE,
    mm::{
        check_ptr, translated_byte_buffer, MapPermission, PTEFlags, PageTable, VirtAddr,
        VirtPageNum,
    },
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        get_syscall_cnt_for_current_task, mmap, munmap, suspend_current_and_run_next,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let bufs = translated_byte_buffer(current_user_token(), ts as *mut u8, size_of::<TimeVal>());
    let tv = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let tv_bytes =
        unsafe { from_raw_parts(&tv as *const TimeVal as *const u8, size_of::<TimeVal>()) };
    let mut offset = 0;
    for buf in bufs {
        let len = buf.len();
        buf.copy_from_slice(&tv_bytes[offset..offset + len]);
        offset += len;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let token = current_user_token();
            let ptr = id as *const u8;
            if check_ptr(token, ptr, 1, PTEFlags::R | PTEFlags::V) {
                let bufs = translated_byte_buffer(token, ptr, 1);
                bufs[0][0] as isize
            } else {
                -1
            }
        }
        1 => {
            let token = current_user_token();
            let ptr = id as *const u8;
            if check_ptr(token, ptr, 1, PTEFlags::W | PTEFlags::V) {
                let mut bufs = translated_byte_buffer(token, ptr, 1);
                bufs[0][0] = data as u8;
                0
            } else {
                -1
            }
        }
        2 => get_syscall_cnt_for_current_task(id) as isize,
        _ => panic!("unreachable code!"),
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    if prot & !0x7 != 0 || prot & 0x7 == 0 {
        return -1;
    }
    let pages = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    //check not mapped
    let start_vpn: VirtPageNum = VirtAddr::from(start).into();
    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    for vpn_num in start_vpn.0..start_vpn.0 + pages {
        let vpn = VirtPageNum(vpn_num);
        if let Some(pte) = page_table.translate(vpn) {
            if pte.is_valid() {
                return -1;
            }
        }
    }
    mmap(
        start.into(),
        pages * PAGE_SIZE,
        MapPermission::from_bits((prot as u8) << 1).unwrap() | MapPermission::U,
    );
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    let pages = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    //check mapped
    let start_vpn: VirtPageNum = VirtAddr::from(start).into();
    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    for vpn_num in start_vpn.0..start_vpn.0 + pages {
        let vpn = VirtPageNum(vpn_num);
        if let Some(pte) = page_table.translate(vpn) {
            if !pte.is_valid() {
                return -1;
            }
        } else {
            return -1;
        }
    }
    munmap(start.into(), pages * PAGE_SIZE);
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
