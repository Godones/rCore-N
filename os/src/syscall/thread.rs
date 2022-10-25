use crate::{mm::kernel_token, task::{add_task, current_task, TaskControlBlock}, trap::{trap_handler, TrapContext}};
use alloc::sync::Arc;
use crate::task::{WAIT_LOCK, WAITTID_LOCK};
pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    let task = current_task().unwrap();

    let process = task.process.upgrade().unwrap();
    // create a new thread
    let new_task = Arc::new(TaskControlBlock::new(
        Arc::clone(&process),
        task.acquire_inner_lock()
            .res
            .as_ref()
            .unwrap()
            .ustack_base,
        true,
    ));
    // debug!("tid: {}", new_task.acquire_inner_lock().res.as_ref().unwrap().tid);
    let new_task_inner = new_task.acquire_inner_lock();
    let new_task_res = new_task_inner.res.as_ref().unwrap();
    let new_task_tid = new_task_res.tid;
    let mut process_inner = process.acquire_inner_lock();
    // add new thread to current process
    let tasks = &mut process_inner.tasks;
    while tasks.len() < new_task_tid + 1 {
        tasks.push(None);
    }
    tasks[new_task_tid] = Some(Arc::clone(&new_task));
    let new_task_trap_cx = new_task_inner.get_trap_cx();
    *new_task_trap_cx = TrapContext::app_init_context(
        entry,
        new_task_res.ustack_top(),
        kernel_token(),
        new_task.kstack.get_top(),
        trap_handler as usize,
    );
    (*new_task_trap_cx).x[10] = arg;
    // add new task to scheduler
    add_task(Arc::clone(&new_task));
    new_task_tid as isize
}

pub fn sys_gettid() -> isize {
    current_task()
        .unwrap()
        .acquire_inner_lock()
        .res
        .as_ref()
        .unwrap()
        .tid as isize
}

/// thread does not exist, return -1
/// thread has not exited yet, return -2
/// otherwise, return thread's exit code
pub fn sys_waittid(tid: usize) -> i32 {
    // warn!("wait tid: {}", tid);
    let task = current_task().unwrap();
    let process = task.process.upgrade().unwrap();
    let wtl = WAITTID_LOCK.lock();
    // warn!("wait tid: {} 2", tid);
    let task_inner = task.acquire_inner_lock();
    let mut process_inner = process.acquire_inner_lock();
    // a thread cannot wait for itself
    if task_inner.res.as_ref().unwrap().tid == tid {
        drop(wtl);
        return -1;
    }
    let mut exit_code: Option<i32> = None;
    let waited_task = process_inner.tasks[tid].as_ref();
    if let Some(waited_task) = waited_task {
        let inner = waited_task.acquire_inner_lock();
        // warn!("wait tid: {} 3", tid);
        if let Some(waited_exit_code) = inner.exit_code {
            exit_code = Some(waited_exit_code);
        }
    } else {
        drop(wtl);
        // waited thread does not exist
        return -1;
    }
    if let Some(exit_code) = exit_code {
        // dealloc the exited thread
        process_inner.dealloc_tid(tid);
        process_inner.tasks[tid] = None;
        drop(wtl);
        exit_code
    } else {
        // warn!("wait tid: {} end", tid);
        drop(wtl);
        // waited thread has not exited
        -2
    }
}