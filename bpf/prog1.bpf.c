// Aggregations map
// struct {
//   __uint(type, BPF_MAP_TYPE_HASH);
//   __type(key, u64);
//   __type(value, u64);
//   __uint(max_entries, (1<<16));
// } map SEC(".maps");
// static __always_inline u64 count_map_entries(struct bpf_map *map, u64 *key,
// u64 *agg, u64 *count) {
//   *count++;
//   return 0;
// }

#include "common.bpf.h"

// Output ringbuf
#define RB_MAX_ENTRIES (1 << 18)
#define SEC_NS (1000000000)
struct {
  __uint(type, BPF_MAP_TYPE_RINGBUF);
  __uint(max_entries, (RB_MAX_ENTRIES * sizeof(u64)));
} rb SEC(".maps");

u64 global_count = 0;

SEC("tp/syscalls/sys_enter_pread64")
u32 pread_query(struct trace_event_raw_sys_enter *ctx) {
  global_count += 1;
  u64 count = global_count;
  if (count >= RB_MAX_ENTRIES) {
    // global_count = RB_MAX_ENTRIES;
    u64 *records = bpf_ringbuf_reserve(&rb, count * sizeof(u64), 0);
    if (records != NULL) {
      bpf_ringbuf_submit(records, 0);
    }
    global_count = 0;
  }
  return 0;
}