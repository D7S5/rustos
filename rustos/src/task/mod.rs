use core::{future::Future, pin::Pin, sync::atomic::AtomicU64};
use alloc::boxed::Box;

pub mod keyboard;
pub mod simple_executor;
pub mod executor;
pub struct Task{
    id: TaskId,
    future: Pin<Box<dyn Future<Output = () >>>,
}

use core::task::{Context, Poll};
impl Task {
    pub fn new(future : impl Future<Output = ()> + 'static) -> Task {
        Task { 
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

//Waker 기능 실행자
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);
use core::sync::atomic::Ordering;
impl TaskId {
    fn new()-> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0); // atomic 사용 한번만 할당
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
        //fetch_add 값을 원자적으로 증가 이전값 반환, 메서드 병렬호출시에도 정확히 한번만 반환
    }
}