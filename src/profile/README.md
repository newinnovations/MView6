# Memory profiling

The `/proc/self/statm` file in Linux provides memory usage statistics for the currently running process.
It contains several fields, each representing different aspects of the process's memory consumption.

1. **Total program size** – The total virtual memory size of the process (in pages).
2. **Resident set size** – The portion of memory currently being used in RAM (in pages).
3. **Shared pages** – The number of pages that are shared with other processes.
4. **Text code size** – The size of the executable code.
5. **Library size** – The size of loaded shared libraries.
6. **Data and stack size** – The size of data segments and stack.
7. **Dirty pages** – The number of pages that have been modified but not written to disk.

Since memory values are reported in pages, you can convert them to bytes by multiplying by the page size (which can be retrieved using `getconf PAGESIZE`).

## Detecting memory leaks

For detecting memory leaks, the most important fields from the MemoryUsage struct are:

* **Resident set size** (`resident_set_size`) – This field represents the portion of the process's memory that is held in RAM.
A continuous increase in this value can indicate a memory leak.

* **Data** (`data`) – This field represents the size of the data segment, which includes initialized and uninitialized data.
Growth in this field can also suggest a memory leak, especially if it keeps increasing without corresponding decreases.

These fields are crucial because they directly reflect the memory usage of your program. Monitoring them over time can help you
identify abnormal increases that may indicate a memory leak.
