#include "common.bpf.h"

// emitted struct
typedef struct {
  u64 time;
  u64 fd;
  u64 cpu;
  u64 count;
} raw_pread_t;

// Output ringbuf
#define RB_MAX_ENTRIES (1024 * sizeof(raw_pread_t))
struct {
	__uint(type, BPF_MAP_TYPE_RINGBUF);
	__uint(max_entries, RB_MAX_ENTRIES);
} ring_buf_pread_query SEC(".maps");

SEC("tp/syscalls/sys_enter_pread64")
u32 pread_query(struct trace_event_raw_sys_enter* ctx) {
  raw_pread_t* q =
      bpf_ringbuf_reserve(&ring_buf_pread_query, sizeof(raw_pread_t), 0);
      if (!q) {
        ERROR("failed to allocate space in ring buffer");
        return 1;
      }
  q->time = bpf_ktime_get_ns();
  q->fd = ctx->args[0];
  q->cpu = bpf_get_smp_processor_id();
  q->count = ctx->args[2];
  bpf_ringbuf_submit(q, 0);
  return 0;
}

// *** LICENSE *** //
char LICENSE[] SEC("license") = "Dual BSD/GPL";
