#pragma once

/**
 * Implement aggregations in eBPF for the query pread_query.
 */

#include "common.bpf.h"
#include "pread_query.bpf.h"

// Depending on group by key, can reduce number of max entries (e.g. for cpu, only need # cpus)
#define AGG_MAX_ENTRIES (16384)

// Since BPF doesn't allow FP, scale values by AVG_SCALE (4 -> +4 sigfigs)
#define AVG_SCALE (1000000)

typedef struct {
  u64 fd;
  u64 cpu;
} group_by_pread_query_t;

// Avg counter for individual item.
typedef struct {
  // Note: the averaged value doesn't have to be u64, but do this to prevent
  // overflows.
  u64 val;
  u64 count;
} avg_t;

// Use val for min/max/count
typedef struct {
  u64 val;
} agg_t;

// Simple aggregations
static __always_inline void max(agg_t *agg, u64 val) {
  if (val > agg->val) agg->val = val;
}
static __always_inline void min(agg_t *agg, u64 val) {
  if (val < agg->val) agg->val = val;
}
static __always_inline void count(agg_t *agg, u64 val) { agg->val += 1; }
static __always_inline void sum(agg_t *agg, u64 val) { agg->val += val; }
static __always_inline void avg(avg_t *agg, u64 val) {
  agg->val += val;
  agg->count += 1;
}

struct {
  __uint(type, BPF_MAP_TYPE_HASH);
  __type(key, group_by_pread_query_t);
  __type(value, agg_t);
  __uint(max_entries, AGG_MAX_ENTRIES);
  __uint(map_flags, BPF_F_NO_PREALLOC);
} count__pread_query SEC(".maps");
struct {
  __uint(type, BPF_MAP_TYPE_HASH);
  __type(key, group_by_pread_query_t);
  __type(value, agg_t);
  __uint(max_entries, AGG_MAX_ENTRIES);
  __uint(map_flags, BPF_F_NO_PREALLOC);
} sum_count_pread_query SEC(".maps");
struct {
  __uint(type, BPF_MAP_TYPE_HASH);
  __type(key, group_by_pread_query_t);
  __type(value, avg_t);
  __uint(max_entries, AGG_MAX_ENTRIES);
  __uint(map_flags, BPF_F_NO_PREALLOC);
} avg_count_pread_query SEC(".maps");

static __always_inline s32 insert_count__pread_query(group_by_pread_query_t key, u64 val) {
  s32 ret;
  agg_t *agg = (agg_t *)bpf_map_lookup_elem(&count__pread_query, &key);
  if (!agg) {
    agg_t init = {val};
    ret = bpf_map_update_elem(&count__pread_query, &key, &init, BPF_NOEXIST);
  } else {
    count(agg, val);
  }
  if (ret != 0) {
    ERROR("failed to insert into count map: %d", ret);
  }
  return ret;
}

typedef struct {
  pread_query_t *buf;
  u64 buf_sz;
  u64 count;
} count__pread_query_ctx_t;

static __always_inline s64 __get_count__pread_query_callback(struct bpf_map *map,
                                                           group_by_pread_query_t *key,
                                                           agg_t *agg,
                                                           count__pread_query_ctx_t *ctx) {
  // Skip if val is 0
  // TODO: migrate from val -> add count to agg_t
  if (agg->val == 0) {
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
  ctx->buf[ctx->count].fd = key->fd;
  ctx->buf[ctx->count].cpu = key->cpu;
  ctx->buf[ctx->count].count_ = agg->val;
  ctx->count += 1;
  return 0;
}

static __always_inline void get_count__pread_query(pread_query_t *buf, u64 buf_sz) {
  count__pread_query_ctx_t ctx = {.buf = buf, .buf_sz = buf_sz, .count = 0};
  bpf_for_each_map_elem(&count__pread_query, __get_count__pread_query_callback, &ctx, 0);
}

static __always_inline u64 __count_count__pread_query_callback(struct bpf_map *map,
                                                             group_by_pread_query_t *key,
                                                             agg_t *agg,
                                                             u64 *count) {
  // Skip if val is 0
  // TODO: migrate from val -> add count to agg_t
  if (agg->val == 0) {
    return 0;
  }
  *count += 1;
  return 0;
}

static __always_inline u64 count_count__pread_query() {
  u64 count = 0;
  bpf_for_each_map_elem(&count__pread_query, __count_count__pread_query_callback, &count, 0);
  return count;
}

static __always_inline u64 __tumble_count__pread_query_callback(struct bpf_map *map,
                                                             group_by_pread_query_t *key,
                                                             agg_t *agg,
                                                             void *ctx) {
  agg->val = 0;
  return 0;
}

static __always_inline void tumble_count__pread_query() {
  bpf_for_each_map_elem(&count__pread_query, __tumble_count__pread_query_callback, NULL, 0);
}

static __always_inline s32 insert_sum_count_pread_query(group_by_pread_query_t key, u64 val) {
  s32 ret;
  agg_t *agg = (agg_t *)bpf_map_lookup_elem(&sum_count_pread_query, &key);
  if (!agg) {
    agg_t init = {val};
    ret = bpf_map_update_elem(&sum_count_pread_query, &key, &init, BPF_NOEXIST);
  } else {
    sum(agg, val);
  }
  if (ret != 0) {
    ERROR("failed to insert into sum map: %d", ret);
  }
  return ret;
}

typedef struct {
  pread_query_t *buf;
  u64 buf_sz;
  u64 count;
} sum_count_pread_query_ctx_t;

static __always_inline s64 __get_sum_count_pread_query_callback(struct bpf_map *map,
                                                           group_by_pread_query_t *key,
                                                           agg_t *agg,
                                                           sum_count_pread_query_ctx_t *ctx) {
  // Skip if val is 0
  // TODO: migrate from val -> add count to agg_t
  if (agg->val == 0) {
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
  ctx->buf[ctx->count].fd = key->fd;
  ctx->buf[ctx->count].cpu = key->cpu;
  ctx->buf[ctx->count].sum_count = agg->val;
  ctx->count += 1;
  return 0;
}

static __always_inline void get_sum_count_pread_query(pread_query_t *buf, u64 buf_sz) {
  sum_count_pread_query_ctx_t ctx = {.buf = buf, .buf_sz = buf_sz, .count = 0};
  bpf_for_each_map_elem(&sum_count_pread_query, __get_sum_count_pread_query_callback, &ctx, 0);
}

static __always_inline u64 __count_sum_count_pread_query_callback(struct bpf_map *map,
                                                             group_by_pread_query_t *key,
                                                             agg_t *agg,
                                                             u64 *count) {
  // Skip if val is 0
  // TODO: migrate from val -> add count to agg_t
  if (agg->val == 0) {
    return 0;
  }
  *count += 1;
  return 0;
}

static __always_inline u64 count_sum_count_pread_query() {
  u64 count = 0;
  bpf_for_each_map_elem(&sum_count_pread_query, __count_sum_count_pread_query_callback, &count, 0);
  return count;
}

static __always_inline u64 __tumble_sum_count_pread_query_callback(struct bpf_map *map,
                                                             group_by_pread_query_t *key,
                                                             agg_t *agg,
                                                             void *ctx) {
  agg->val = 0;
  return 0;
}

static __always_inline void tumble_sum_count_pread_query() {
  bpf_for_each_map_elem(&sum_count_pread_query, __tumble_sum_count_pread_query_callback, NULL, 0);
}

static __always_inline s32 insert_avg_count_pread_query(group_by_pread_query_t key, u64 val) {
  s32 ret;
  avg_t *agg = (avg_t *)bpf_map_lookup_elem(&avg_count_pread_query, &key);
  if (!agg) {
    avg_t init = {val, 1};
    ret = bpf_map_update_elem(&avg_count_pread_query, &key, &init, BPF_NOEXIST);
  } else {
    avg(agg, val);
  }
  if (ret != 0) {
    ERROR("failed to insert into avg map: %d", ret);
  }
  return ret;
}

typedef struct {
  pread_query_t *buf;
  u64 buf_sz;
  u64 count;
} avg_count_pread_query_ctx_t;

static __always_inline s64 __get_avg_count_pread_query_callback(struct bpf_map *map,
                                                           group_by_pread_query_t *key,
                                                           avg_t *agg,
                                                           avg_count_pread_query_ctx_t *ctx) {
  // Skip if val is 0
  // TODO: migrate from val -> add count to agg_t
  if (agg->val == 0) {
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
  ctx->buf[ctx->count].fd = key->fd;
  ctx->buf[ctx->count].cpu = key->cpu;
  ctx->buf[ctx->count].avg_count = agg->val;
  // Defer computation until here
  ctx->buf[ctx->count].avg_count /= agg->count;
  // ctx->buf[ctx->count].avg_count_count = agg->count;
  ctx->count += 1;
  return 0;
}

static __always_inline void get_avg_count_pread_query(pread_query_t *buf, u64 buf_sz) {
  avg_count_pread_query_ctx_t ctx = {.buf = buf, .buf_sz = buf_sz, .count = 0};
  bpf_for_each_map_elem(&avg_count_pread_query, __get_avg_count_pread_query_callback, &ctx, 0);
}

static __always_inline u64 __count_avg_count_pread_query_callback(struct bpf_map *map,
                                                             group_by_pread_query_t *key,
                                                             avg_t *agg,
                                                             u64 *count) {
  // Skip if val is 0
  // TODO: migrate from val -> add count to agg_t
  if (agg->val == 0) {
    return 0;
  }
  *count += 1;
  return 0;
}

static __always_inline u64 count_avg_count_pread_query() {
  u64 count = 0;
  bpf_for_each_map_elem(&avg_count_pread_query, __count_avg_count_pread_query_callback, &count, 0);
  return count;
}

static __always_inline u64 __tumble_avg_count_pread_query_callback(struct bpf_map *map,
                                                             group_by_pread_query_t *key,
                                                             avg_t *agg,
                                                             void *ctx) {
  agg->val = 0;
  agg->count = 0;
  return 0;
}

static __always_inline void tumble_avg_count_pread_query() {
  bpf_for_each_map_elem(&avg_count_pread_query, __tumble_avg_count_pread_query_callback, NULL, 0);
}

