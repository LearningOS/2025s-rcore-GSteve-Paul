# 第四章 练习

## 实现功能

### 迁移 sys_get_time, sys_mmap, sys_munmap

#### sys_get_time

不需要调整

#### sys_mmap 和 sys_munmap

将处理系统调用的核心代码转移到了`src/task/task.rs`，并利用`current_task()`函数得到当前的任务控制块。

### 实现 sys_spawn

与`sys_fork`相似，但是不能复制父进程的地址空间，而是利用`MemorySet::from_elf(elf_data);`方法获取这个程序的地址空间、用户栈和程序入口。

然后在内核空间申请内核栈，并初始化好`TaskContext`和`TrapContext`，构造`TaskControlBlock`，最后在它的父进程里添加上这个子进程。

### 实现 stride 调度算法

#### 实现 sys_set_priority

在`TaskControlBlock`中添加一个字段`priority`表示优先级

#### 实现 P.pass

因为有 $P.pass=\frac{BigStride}{P.priority}$ ，为了能更加精确，应该让$BigStride \mid P.priority$。因此令$BigStride = 16 \cdot 15 \cdot 14 \cdot 13 \cdot 11 \cdot 9$。

所以在`TaskControlBlock`中又添加一个字段`stride`表示进程已经运行的长度，然后定义一个函数`add_pass()`封装增加`stride`的过程

#### 实现调度

把`TaskManager`中维护就绪进程的数据结构换成了一个小根堆，为`TaskControlBlock`按照进程号和`stride`实现了`PartialEq` `Eq` `PartialOrd` `Ord`，使得这个小根堆可以维护`stride`最小的进程控制块。

在`run_tasks()`函数获取到应该调度的进程后，调用`add_pass()`增加`stride`

## 简答问题

### 简答1

#### 1.1

不是，因为无符号溢出，$(250 + 10) \pmod{256} = 4 < 255$

#### 1.2

在第一次调度时，所有$Stride$都是$0$，所以一定满足

设某次调度前，有最大次小最小$Stride$即$StrideMax \ge StrideSecondMin \ge StrideMin \land StrideMax - StrideMin \le \frac{BigStride}{2}$。

调度后，$StrideMin$会变成$StrideMinNew = StrideMin + \frac{BigStride}{prio}$。

如果$StrideMinNew > StrideMax$，那么最大最小之差为$StrideMin - StrideMax + \frac{BigStride}{prio} \le \frac{BigStride}{prio} \le \frac{BigStride}{2}$

如果$StrideMinNew \le StrideMax$，那么最大最小之差为$StrideMin - StrideSecondMin \le StrideMin - StrideMin \le \frac{BigStride}{2}$

根据数学归纳法，所以得证

#### 1.3

我们让$BigStride \le 存储Stride的无符号整数的最大值MAX$，那么溢出后的$StrideSecondMin - StrideMinNew = StrideMin - StrideMinNew + StrideSecondMin - StrideMin = MAX -pass + StrideSecondMin - StrideMin \ge \frac{MAX}{2} + StrideSecondMin - StrideMin \ge \frac{MAX}{2}$

因此如果二者之差小于$\frac{MAX}{2}$，还是正常顺序，否则就要反过来。
```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let abs = if self.0 < other.0 {
            other.0 - self.0
        } else {
            self.0 - other.0
        };
        if abs < u64::MAX / 2 {
            self.0.partial_cmp(&other.0)
        } else {
            Reverse(self.0).partial_cmp(&Reverse(other.0))
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
```

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。