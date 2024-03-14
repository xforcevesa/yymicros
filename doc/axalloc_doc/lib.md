# 内存页表分配
> 官网是这样介绍：全局内存分配器，提供了`GlobalAllocator`，它是`core::alloc::GlobalAlloc`trait的实现。一个静态全局变量类型`GlobalAllocator`在定义时使用了`global_allocator`属性，注册为标准库的默认分配器

## cfg_if
功能：挑选默认的字节分配器  
如果开启slab特性，就使用SlabByteAllocator作为默认的字节分配器  
如果开启buddy特性，就使用BuddyByteAllocator作为默认的字节分配器  
如果开启tlsf特性，就使用TlsfByteAllocator作为默认的字节分配器  

## GlobalAllocator 全局内存分配器
- balloc: 默认字节分配器分配，小粒度内存分配器
- palloc：大粒度内存分配器，分配粒度可能为PAGE_SIZE

## GlobalAllocator的一些成员函数
### new
构造函数

### name
返回balloc的内存分配器名字。静态生命周期的字符串。

### init
> 用给定的区域初始化分配器。它首先将整个区域添加到页分配器，然后分配一个小区域(32 KB)来初始化字节分配器。因此，给定的区域必须大于32kb。

### add_memory
> 将给定区域交给分配器，该函数会将整个区域添加至字节分配器

### alloc
> 分配任意数量的字节。  

for balloc  

返回已分配区域的左边界。
- 它首先尝试从字节分配器进行分配。
- 如果没有内存，它会向页分配器请求更多内存，并将其添加到字节分配器。
' align_pow2 '必须是2的幂，并且返回的区域绑定将与它对齐。

### dealloc
> 将分配的空间还给字节分配器  

for balloc

### alloc_pages
> 分配连续的页  
for palloc  
> 从页分配器中分配`num_pages`个页

### dealloc_pages
> 将那些从`pos`这个位置起始的页 还给页分配器  

for palloc

### used_bytes
> 返回字节分配器中已经分配的字节的数量

### available_bytes
> 返回字节分配器中可用字节的数量。

### used_pages
> 返回页分配器中已经分配的页的数量。

### available_pages
> 返回也分配器中可用页的数量。


## impl GlobalAlloc for GlobalAllocator
对于GlobalAllocator，要实现GlobalAlloc这些接口。  
相当于GlobalAlloc是抽象类（trait）

### alloc
尝试调用结构体GlobalAllocator的alloc字节分配器，成功则转换为`*mut u8`类型的可变指针；失败则处理错误。

### dealloc
和上面相反。尝试调用全局内存分配器的`dealloc`来回收字节。

-----
与此同时，还声明了一个静态全局内存分配器GLOBAL_ALLOCATOR

-----

## global_allocator
外部函数，调用时会返回刚刚声明的那个内存分配器的引用

## global_init
外部函数
> 用给定的内存区域初始化内存分配器。  
请注意，内存区域边界只是数字，分配器并不实际访问该区域。用户应该确保该区域是有效的，并且没有被其他人使用，这样分配的内存也是有效的。  
这个函数应该只被调用一次，并且在任何分配之前。

## global_add_memory
外部函数
> 将给定的内存区域添加到全局分配器。用户应该确保该区域是有效的，并且没有被其他人使用，这样分配的内存也是有效的。它类似于上面的`global_init`，但可以多次调用。
