use alloc::sync::Arc;
use hal::{constant::{Constant, ConstantsHal}, println, trap::{TrapContext, TrapContextHal}};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use log::{debug, info, trace};
use crate::{syscall::SysError, timer::get_current_time_duration, trap::user_trap_handler};
use crate::task::TaskControlBlock;
use crate::executor;
use crate::utils::async_utils::{get_waker,suspend_now};
use crate::processor::processor::*;
use crate::trap::trap_return;
use super::task::TaskStatus;
use crate::processor::{context::EnvContext,processor::current_processor};

/// The outermost future for user task
pub struct UserTaskFuture <F: Future + Send + 'static>{
    /// pub for cpu_mask
    pub task: Arc<TaskControlBlock>,
    env: EnvContext,
    future: F,
}
/// The outermost future for kernel task
pub struct KernelTaskFuture<F: Future<Output = ()> + Send + 'static> {
    env: EnvContext,
    future: F,
}

impl <F: Future + Send + 'static> UserTaskFuture <F> {
    #[inline]
    /// new a user task future
    pub fn new(task: Arc<TaskControlBlock>, future: F) -> Self {
        Self {
            task,
            env: EnvContext::new(),
            future,
        }
    }
}

impl <F:Future+Send+'static> Future for UserTaskFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        //info!("now poll task {}", self.task.tid());
        let this = unsafe {self.get_unchecked_mut()};
        switch_to_current_task(current_processor(),&mut this.task,&mut this.env);
        let ret = unsafe{Pin::new_unchecked(&mut this.future).poll(cx)};
        //info!("switch out current task, current task is {}", current_task().unwrap().tid());
        switch_out_current_task(current_processor(),&mut this.env);
        ret
    }
}

impl<F: Future<Output = ()> + Send + 'static> KernelTaskFuture<F> {
    /// new a kernel task future
    pub fn new(future: F) -> Self {
        Self {
            env: EnvContext::new(),
            future,
        }
    }
}
impl<F: Future<Output = ()> + Send + 'static> Future for KernelTaskFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        switch_to_current_kernel(current_processor(),&mut this.env);
        let ret = unsafe { Pin::new_unchecked(&mut this.future).poll(cx) };
        switch_to_current_kernel(current_processor(),&mut this.env);
        ret
    }
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process 
pub async fn run_tasks(task: Arc<TaskControlBlock>) {  
    //info!("into run_tasks");
    task.set_waker(get_waker().await);
    /*info!(
        "into task loop, sepc {:#x}, trap cx addr {:#x}",
        current_task().unwrap().inner_exclusive_access().get_trap_cx().sepc,
        current_task().unwrap().inner_exclusive_access().get_trap_cx() as *const TrapContext as usize,
    );*/
    let mut is_interrupted = false;
    loop {
        // check current task status before return
        match task.get_status() {
            TaskStatus::Zombie => break,
            TaskStatus::Stopped => suspend_now().await,
            _ => {}
        }

        // return to user space and return back from user space
        trap_return(&task, is_interrupted);

        // task status might be change by other task
        match task.get_status() {
            TaskStatus::Zombie => break,
            TaskStatus::Stopped => suspend_now().await,
            _ => {}
        }

        let cx = task.get_trap_cx();
        let old_a0 = cx.ret_nth(0);
        // back from user space
        is_interrupted = user_trap_handler().await;

        // check current task status after return
        // task status maybe already change
        match task.get_status() {
            TaskStatus::Zombie => break,
            TaskStatus::Stopped => suspend_now().await,
            _ => {}
        }

        task.check_and_handle(is_interrupted, old_a0);
    }
}

/// spawn a new async user task
pub fn spawn_user_task(task: Arc<TaskControlBlock>){
    //info!("now in spawn_user_task");
    let future = UserTaskFuture::new(task.clone(), run_tasks(task));
    let (runnable, user_task) = executor::spawn(future);
    runnable.schedule();
    user_task.detach();
}

///spawn a new async kernel task, doing for kernel init work such as initproc
pub fn spawn_kernel_task<F: Future<Output = ()> + Send + 'static>(kernel_task: F) {
    //info!("now in spawn_kernel_task");
    let future = KernelTaskFuture::new(kernel_task);
    let (runnable, task) = executor::kernel_spawn(future);
    runnable.schedule();
    task.detach();
}
