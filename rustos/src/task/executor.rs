use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc};
use core::{task::Waker};
use crossbeam_queue::ArrayQueue;

pub struct Executor {
    tasks : BTreeMap<TaskId, Task>,
    task_queue : Arc<ArrayQueue<TaskId>>, 
    // Arc 참조카운팅을 통해 여러 소유자 간 가치 소유권 공유가능
    //할당 ==0 값 필요 x 할당취소
    waker_cache: BTreeMap<TaskId, Waker>,
    //반복된 작업의 깨우기일 경우 캐시를 이용
    //깨운 작업 TaskId 대기열로 push
}

use core::task::{Context, Poll};
impl Executor {
    pub fn new() -> Self {
        Executor { 
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)), // 100 , 시스템 task 수
            waker_cache: BTreeMap::new()
        }
    }
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with the same id already exists");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
            } = self;
            while let Ok(task_id) = task_queue.pop() {
                let task = match tasks.get_mut(&task_id) {
                    Some(task) => task,
                    None => continue
                };
            let waker = waker_cache
            .entry(task_id)
            .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
            }
    }
    pub fn run(&mut self) -> ! { //Loop 반환값
        loop {
            self.run_ready_tasks(); // 준비완료시 실행
            self.sleep_if_idle(); //절전모드
        }
    }
    //if sleeped interrupts race condition check
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};
        interrupts::disable(); //interrupts disable 
        if self.task_queue.is_empty() {
            enable_and_hlt(); //check 
        } else{
            interrupts::enable(); //int
            // interrupts enable
        }
    }
}

//TaskWaker 사용하려면  인스턴스로 변경하여야함.Wake 구현
struct TaskWaker {
    task_id : TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>> ) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }
}
// 인스턴스의 호환문제로 Trait Wake의 형식을 따라야함
use alloc::task::Wake;
impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}