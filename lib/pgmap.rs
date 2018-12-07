use serde_json::Value;

// See `src/mon/PGMap.h` in ceph's source
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct PGMap {
    pub osd_stats_sum: OsdStatsSum,
    pub pg_stats_delta: PgStatsDelta,
    pub min_last_epoch_clean: Option<i32>,
    pub stamp: String,
    pub pg_stats_sum: PgStatsSum,
    pub last_pg_scan: i32,
    pub full_ratio: Value,
    pub pool_stats: Vec<PoolStats>,
    pub version: i32,
    pub last_osdmap_epoch: i32,
    pub near_full_ratio: Value,
    pub osd_stats: Vec<OsdStats>,
    pub pg_stats: Vec<PgStats>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct OsdStats {
    pub snap_trim_queue_len: i32,
    pub kb: i32,
    pub fs_perf_stat: Option<FsPerfStat>,
    pub hb_in: Option<Vec<i32>>,
    pub num_snap_trimming: i32,
    pub hb_out: Option<Vec<i32>>,
    pub kb_avail: i32,
    pub kb_used: i32,
    pub op_queue_age_hist: OpQueueAgeHist,
    pub osd: i32,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct PgStatsDelta {
    pub acting: Option<i32>,
    pub log_size: i32,
    pub ondisk_log_size: i32,
    pub stat_sum: StatSum,
    pub up: Option<i32>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct StatSum {
    pub num_evict: Option<i32>,
    pub num_evict_kb: Option<i32>,
    pub num_bytes_hit_set_archive: Option<i32>,
    pub num_whiteouts: i32,
    pub num_objects_pinned: Option<i32>,
    pub num_scrub_errors: i32,
    pub num_evict_mode_full: Option<i32>,
    pub num_read: i32,
    pub num_objects_recovered: i32,
    pub num_objects_omap: i32,
    pub num_objects_missing_on_primary: i32,
    pub num_write: i32,
    pub num_object_clones: i32,
    pub num_objects: i32,
    pub num_deep_scrub_errors: i32,
    pub num_shallow_scrub_errors: i32,
    pub num_read_kb: i32,
    pub num_objects_missing: Option<i32>,
    pub num_flush_kb: Option<i32>,
    pub num_flush_mode_high: Option<i32>,
    pub num_write_kb: i32,
    pub num_evict_mode_some: Option<i32>,
    pub num_objects_degraded: i32,
    pub num_flush: Option<i32>,
    pub num_objects_misplaced: Option<i32>,
    pub num_bytes_recovered: i32,
    pub num_objects_hit_set_archive: i32,
    pub num_keys_recovered: i32,
    pub num_flush_mode_low: Option<i32>,
    pub num_objects_unfound: i32,
    pub num_promote: Option<i32>,
    pub num_object_copies: i32,
    pub num_bytes: i32,
    pub num_objects_dirty: i32,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct PgStatsSum {
    pub acting: Option<i32>,
    pub log_size: i32,
    pub ondisk_log_size: i32,
    pub stat_sum: StatSum,
    pub up: Option<i32>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct OsdStatsSum {
    pub snap_trim_queue_len: i32,
    pub kb: i32,
    pub fs_perf_stat: Option<FsPerfStat>,
    pub hb_in: Option<Vec<i32>>,
    pub num_snap_trimming: i32,
    pub hb_out: Option<Vec<i32>>,
    pub kb_avail: i32,
    pub kb_used: i32,
    pub op_queue_age_hist: OpQueueAgeHist,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct OpQueueAgeHist {
    pub upper_bound: i32,
    pub histogram: Vec<i32>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct FsPerfStat {
    pub apply_latency_ms: i32,
    pub commit_latency_ms: i32,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct PoolStats {
    pub log_size: i32,
    pub ondisk_log_size: i32,
    pub up: Option<i32>,
    pub acting: Option<i32>,
    pub poolid: i32,
    pub stat_sum: StatSum,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct PgStats {
    pub last_scrub: String,
    pub last_clean_scrub_stamp: String,
    pub parent_split_bits: i32,
    pub last_active: String,
    pub pin_stats_invalid: Option<bool>,
    pub reported_epoch: String,
    pub log_start: String,
    pub log_size: i32,
    pub hitset_stats_invalid: Option<bool>,
    pub stats_invalid: Value,
    pub acting_primary: i32,
    pub reported_seq: String,
    pub ondisk_log_size: i32,
    pub mapping_epoch: i32,
    pub dirty_stats_invalid: Option<bool>,
    pub state: String,
    pub version: String,
    pub last_became_peered: Option<String>,
    pub last_undegraded: Option<String>,
    pub pgid: String,
    pub parent: String,
    pub acting: Vec<i32>,
    pub up_primary: i32,
    pub last_fullsized: Option<String>,
    pub last_epoch_clean: i32,
    pub last_deep_scrub_stamp: String,
    pub stat_sum: StatSum,
    pub last_deep_scrub: String,
    pub last_fresh: String,
    pub last_scrub_stamp: String,
    pub created: i32,
    pub up: Vec<i32>,
    pub hitset_bytes_stats_invalid: Option<bool>,
    pub last_peered: Option<String>,
    pub last_became_active: String,
    pub omap_stats_invalid: Option<bool>,
    pub last_clean: String,
    pub last_unstale: String,
    pub last_change: String,
    pub blocked_by: Option<Vec<i32>>,
    pub ondisk_log_start: String,
}

#[cfg(test)]
mod tests {
    use super::PGMap;
    use crate::from::FromFile;

    // Jewel tests
    #[test]
    #[should_panic]
    fn pgmap_from_jewel_file_panic() {
        let pgmap = PGMap::from_file("test/jewel/pg_dump_safe.json").unwrap();
        // An OSD is safe to remove so up should not be 0
        assert_eq!(pgmap.pg_stats.first().unwrap().up.len(), 0);
    }

    #[test]
    fn pgmap_from_jewel_file() {
        let pgmap = PGMap::from_file("test/jewel/pg_dump_safe.json").unwrap();
        // First pg_stat.up should be length 3
        assert_eq!(pgmap.pg_stats.first().unwrap().up.len() as i32, 3);
    }

    #[test]
    #[should_panic]
    fn pgmap_from_jewel_file_no_osd_panic() {
        let pgmap = PGMap::from_file("test/jewel/pg_dump_no_osd.json").unwrap();
        // First pg_stat.state should be "creating" since there are no OSDs
        assert_eq!(
            pgmap.pg_stats.first().unwrap().state,
            "active+clean".to_owned()
        );
    }

    #[test]
    fn pgmap_from_jewel_file_no_osd() {
        let pgmap = PGMap::from_file("test/jewel/pg_dump_no_osd.json").unwrap();
        // First pg_stat.up should be length 0
        assert_eq!(pgmap.pg_stats.first().unwrap().up.len() as i32, 0);
    }

    #[test]
    fn pgmap_from_jewel_file_non_safe() {
        let pgmap = PGMap::from_file("test/jewel/pg_dump_non_safe.json").unwrap();
        // Up should be 2 as to not meet min_size + 1, as such acting should be 2
        assert_eq!(pgmap.pg_stats.first().unwrap().up.len() as i32, 2);
        assert_eq!(pgmap.pg_stats.first().unwrap().acting.len() as i32, 2);
    }

    #[test]
    #[should_panic]
    fn pgmap_from_jewel_file_non_safe_panic() {
        let pgmap = PGMap::from_file("test/jewel/pg_dump_non_safe.json").unwrap();
        assert_eq!(pgmap.pg_stats.first().unwrap().up.len() as i32, 3);
    }

    // Firefly tests
    #[test]
    fn pgmap_from_firefly_file() {
        let pgmap = PGMap::from_file("test/firefly/pg_dump_safe.json").unwrap();
        assert_eq!(pgmap.pg_stats.first().unwrap().up.len() as i32, 3);
    }

    #[test]
    #[should_panic]
    fn pgmap_from_firefly_file_panic() {
        let pgmap = PGMap::from_file("test/firefly/pg_dump_safe.json").unwrap();
        assert_eq!(pgmap.pg_stats.first().unwrap().acting.len() as i32, 0);
    }

    #[test]
    #[should_panic]
    fn pgmap_from_ceph_panic() {
        use crate::from::FromCeph;
        let pgmap = PGMap::from_ceph("pg dump");
        assert_eq!(pgmap.is_ok(), true);
    }
}
