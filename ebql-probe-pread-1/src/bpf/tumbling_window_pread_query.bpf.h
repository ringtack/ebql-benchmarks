#pragma once

/**
 * Windowing capabilities that turn eBPF event streams into bounded relations.
 * Three window types are supported:
 * - Count(N, step): Stores a window of N elements, with a step <= N.
 * - Time(Interval, step): Stores a window of interval time span, with step <=
 * interval.
 * - Session(threshold): Stores a window by sessions of activity, with an
 * inactivity threshold.
 *
 * Stream processing occurs only when the step is triggered (e.g. the step
 * duration elapsed in a time interval).
 *
 * RESTRICTIONS (until I can figure out more verifier stuff):
 * - For counts, WINDOW_SIZE % STEP == 0 (i.e. WINDOW_SIZE must be divisible by
 * STEP)
 * - For time, STEP == INTERVAL (i.e. all time windows must be tumbling
 * windows).
 */

#include "common.bpf.h"
#include "pread_query.bpf.h"

#define INTERVAL (1000000000)

// Window representation: for tumbling windows over the aggregations currently supported, only need
// count/time to know when to tumble.
typedef struct window {
  u64 start_time;
} window_t;

window_t w = {0};

/**
 * Adds to window. Returns whether flushing is needed
 */
static __always_inline bool window_add(u64 time) {
  if (w.start_time == 0) {
    w.start_time = time;
    return false;
  }
  return (w.start_time + INTERVAL < time);
}

/**
 * Tumbles the window.
 */
static __always_inline void window_tumble(u64 time) { w.start_time = time; }
