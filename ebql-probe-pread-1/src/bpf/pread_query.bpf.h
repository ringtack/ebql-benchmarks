#pragma once
// *** HEADER FOR QUERY select_0IjMb *** //
#include "common.bpf.h" /* common definitions */

// *** MACRO DEFINITIONS *** //


// *** STRUCT DEFINITIONS *** //
typedef struct {
	u64 fd;
	u64 cpu;
	u64 count_;
	u64 sum_count;
	u64 avg_count;
} pread_query_t;


// *** GLOBAL DEFINITIONS *** //


