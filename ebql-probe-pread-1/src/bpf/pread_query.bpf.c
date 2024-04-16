// *** SOURCE FOR pread_query *** //

// *** INCLUDES SECTION *** //
#include "pread_query.bpf.h" /* pread_query's definitions */
#include "tumbling_window_pread_query.bpf.h" /* External includes (tumbling_window) */
#include "agg_pread_query.bpf.h" /* External includes (agg) */


// *** MAPS SECTION *** //
struct {
	__uint(type, BPF_MAP_TYPE_RINGBUF);
	__uint(max_entries, 4194280);
} ring_buf_pread_query SEC(".maps");


// *** CODE SECTION *** //
SEC("tp/syscalls/sys_enter_pread64")
u32 pread_query(struct trace_event_raw_sys_enter* ctx) {
	u64 time;
	TIME(time);
	u64 fd;
	fd = ctx->args[0];
	u64 cpu;
	CPU(cpu);
	u64 count;
	count = ctx->args[2];
	DEBUG("Got event");
	bool tumble = window_add(time);
	if (tumble) {
		u64 n_results = count_count__pread_query();
		if (n_results >= 104857) {
			WARN("Got too many results; truncating to max rb entries...");
			n_results = 104857;
		}
		if (n_results > 0) {
			pread_query_t* buf = bpf_ringbuf_reserve(&ring_buf_pread_query, n_results * sizeof(pread_query_t), 0);
			if (!buf) {
				ERROR("Failed to allocate from ring buffer");
				return 1;
			}
			get_count__pread_query(buf, n_results);
			get_sum_count_pread_query(buf, n_results);
			get_avg_count_pread_query(buf, n_results);
			bpf_ringbuf_submit(buf, 0);
		}
		tumble_count__pread_query();
		tumble_sum_count_pread_query();
		tumble_avg_count_pread_query();
		window_tumble(time);
	}
	insert_count__pread_query((group_by_pread_query_t){fd, cpu}, 1);
	insert_sum_count_pread_query((group_by_pread_query_t){fd, cpu}, count);
	insert_avg_count_pread_query((group_by_pread_query_t){fd, cpu}, count);
	return 0;
}


// *** LICENSE *** //
char LICENSE[] SEC("license") = "Dual BSD/GPL";
