# 页表实现
一个RAII（资源获取即初始化）包装器包装连续大小为4K的页表。

## 结构体GlobalPage
- start_vaddr：开始的虚拟地址，类型为VirtAddr，也就是usize
- num_pages: 页的数量。

## GlobalPage的功能实现
### alloc
**功能：分配一个4K大小的页面**
通过全局分配器的方式分配1个4K大小的页面  
然后将返回的虚拟地址包装成`GlobalPage`的实例，并返回一个结果

### alloc_zero
功能：分配一个4K大小的页面，并填充全0  
通过调用上面`alloc`函数来分配页面  
使用`zero`方法填充0  

### alloc_contiguous
功能：分配连续大小为4K的多个页面  
通过全局分配器`global_allocator`  
指定页面数量和对其方式来分配页面  
返回页开头虚拟地址  

### start_vaddr
功能：返回页开头的虚拟地址

### start_paddr
功能：返回页开头的物理地址（由虚拟地址转换而来）

### size
功能：获取多个页占用的内存空间大小

### as_ptr
功能：将页面转换为不可变的原始指针

### as_mut_ptr
功能：将页面转换为可变的原始指针

### fill
功能：用指定的字节填充页面

### zero
功能：用0填充页面

### as_slice
功能：将页面转换为不可变的切片

### as_slice_mut
功能：将页面转换为可变的切片数据。

## Drop： impl Drop for GlobalPage
功能：GlobalPage的析构函数，释放页面

## alloc_err_to_ax_err
功能：将AllocError类型的错误转换成AxError类型的错误


