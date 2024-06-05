// *** SOURCE FOR pread_query *** //

#include "common.bpf.h"
#include "pread_query.bpf.h"

// Group by key
typedef struct {
  u64 fd;
  u64 cpu;
} group_by_pread_query_t;

// Aggregations
typedef struct {
  // Max
  u64 max;
  // Summed value for average
  u64 val;
  // Counts
  u64 count;
} agg_t;

// Aggregations map
struct {
  __uint(type, BPF_MAP_TYPE_HASH);
  __type(key, group_by_pread_query_t);
  __type(value, agg_t);
  __uint(max_entries, (1<<16));
} aggs_pread_query SEC(".maps");

// Output ringbuf
#define RB_MAX_ENTRIES (4194280)
struct {
	__uint(type, BPF_MAP_TYPE_RINGBUF);
	__uint(max_entries, RB_MAX_ENTRIES);
} ring_buf_pread_query SEC(".maps");

// A second in NS
#define SEC_NS (1000000000)
// Record last seen value timestamp
u64 last_time = 0;
u64 gb_count = 0;


// Helper callback to clear aggregations map
static __always_inline u64 __clear_aggs_pread_query_callback(struct bpf_map *map,
                                                             group_by_pread_query_t *key,
                                                             agg_t *agg,
                                                             void *ctx) {
  agg->max = 0;
  agg->val = 0;
  agg->count = 0;
  return 0;
}

// Helper callback to get aggregations
typedef struct {
  pread_query_t *buf;
  u64 buf_sz;
  u64 count;
} ctx_t;

static __always_inline u64 __get_aggs_pread_query_callback(struct bpf_map *map, group_by_pread_query_t *key, agg_t *agg, ctx_t *ctx) {
  // Skip if count is 0
  if (agg->count == 0) {
    return 0;
  }
  // Set agg value
  if (!ctx || !ctx->buf) {
    ERROR("Passed null context/context buffer in");
    return 1;
  }
  if (ctx->count >= ctx->buf_sz) {
    WARN("Number of aggregation results exceeds buf size; stopping...");
    return 1;
  }
  // Defer computation until here
  ctx->buf[ctx->count].fd = key->fd;
  ctx->buf[ctx->count].cpu = key->cpu;
  ctx->buf[ctx->count].avg_count = agg->val / agg->count;
  ctx->buf[ctx->count].max_count = agg->max;
  ctx->count += 1;
  return 0;
}

// Helper callback to count number of values
static __always_inline u64
__count_aggs_pread_query_callback(struct bpf_map *map, group_by_pread_query_t *key, agg_t *agg,
                                  u64 *count) {
  if (agg->val == 0) {
    return 0;
  }
  *count += 1;
  return 0;
}

SEC("tp/syscalls/sys_enter_pread64")
u32 pread_query(struct trace_event_raw_sys_enter* ctx) {
  u64 time = bpf_ktime_get_ns();
  u64 fd = ctx->args[0];
  u64 cpu = bpf_get_smp_processor_id();
  u64 count = ctx->args[2];

  // Check if need to reset second timer
  bool reset = false;
  if (last_time == 0) {
    last_time = time;
  } else if ((time - last_time) > SEC_NS) {
    reset = true;
  }

  if (reset) {
    // Get # of unique values
    u64 count = gb_count;
    // Need to use for each map elem since BPF for some reason doesn't allow non-constants
    // (i.e. if I tried using `gb_count`), but does allow something computed like this...
    // bpf_for_each_map_elem(&aggs_pread_query, __count_aggs_pread_query_callback, &count, 0);
    if (count > 0) {
      if (count >= (RB_MAX_ENTRIES / sizeof(pread_query_t))) {
        count = (RB_MAX_ENTRIES / sizeof(pread_query_t));
      }
      pread_query_t* buf =
          bpf_ringbuf_reserve(&ring_buf_pread_query, count * sizeof(pread_query_t), 0);
      if (!buf) {
        ERROR("Failed to allocate from ring buffer");
        return 1;
      }
      // Create result values
      ctx_t ctx = {
        .buf = buf,
        .buf_sz = count,
        .count = 0,
      };
      bpf_for_each_map_elem(&aggs_pread_query, __get_aggs_pread_query_callback, &ctx, 0);
      bpf_ringbuf_submit(buf, 0);
    }

    // Clear aggs map
    bpf_for_each_map_elem(&aggs_pread_query, __clear_aggs_pread_query_callback, 0, 0);
    // Update counters
    last_time = time;
    gb_count = 0;
	}

  // Insert aggregations
  group_by_pread_query_t gb = {fd, cpu};
  agg_t* agg = bpf_map_lookup_elem(&aggs_pread_query, &gb);
  // If non-existent, insert
  if (!agg) {
    agg_t agg = {
        .max = count,
        .val = count,
        .count = 1,
    };
    s64 res = bpf_map_update_elem(&aggs_pread_query, &gb, &agg, BPF_NOEXIST);
    if (!res) {
      ERROR("failed to update map elem: %lld", res);
    }
    // Update total count
    gb_count += 1;
  } else {
    // Check if count == 0; in that case, reset occurred and map was cleared, so update count
    gb_count += (agg->count == 0) ? 1 : 0;

    // Update aggregate values
    agg->max = (agg->max < count) ? count : agg->max;
    agg->val += count;
    agg->count += 1;
  }

	return 0;
}

// *** LICENSE *** //
char LICENSE[] SEC("license") = "Dual BSD/GPL";
