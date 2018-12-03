extern crate ansi_term;
extern crate serde_json;

use ansi_term::Colour;

use error::CSDError;
use from::*;
use osdmap::OsdMap;
use pgmap::*;
use pgstate::*;

use std::collections::BinaryHeap;
use std::fmt;

// Format for printing
#[derive(Clone, Copy, Debug)]
pub enum Format {
    Pretty,
    Json,
}

// The removability status of an OSD. Using an enum for precedence:
// Safe < Unknown < NonSafe
#[derive(Serialize, Debug, Copy, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub enum Status {
    Safe,
    Unknown,
    NonSafe,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Status::Unknown => write!(f, "Pending"),
            Status::Safe => write!(f, "Removable"),
            Status::NonSafe => write!(f, "Not removable"),
        }
    }
}

pub struct PgDiag {
    osd_id: i32,
    pg_info: PgInfo,
}

impl PgDiag {
    fn new(osd_id: i32, pg_info: PgInfo) -> PgDiag {
        PgDiag { osd_id, pg_info }
    }
}

// Holds information about a PG's status, it's ID and state
#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub struct PgInfo {
    pg_id: String,
    pg_state: String,
    rm_safety: RmSafety,
}

impl PgInfo {
    fn new(states: &str, pgid: String) -> PgInfo {
        PgInfo {
            pg_id: pgid,
            pg_state: states.to_string(),
            rm_safety: RmSafety::new(&states),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OsdDiag {
    osd_id: i32,
    osd_status: BinaryHeap<Status>,
}

impl OsdDiag {
    fn new(osd_id: i32) -> OsdDiag {
        OsdDiag {
            osd_id,
            osd_status: BinaryHeap::new(),
        }
    }
}

// Used to print ClusterDiag in a nicer way. Since ClusterDiag.osd_diags use
// binary heaps to order status priority then it is very inconvenient for
// printing as JSON
#[derive(Serialize, Default)]
pub struct ClusterReview {
    #[serde(rename = "Removable")]
    removable: Vec<i32>,
    #[serde(rename = "Not Removable")]
    not_removable: Vec<i32>,
    #[serde(rename = "Pending")]
    pending: Vec<i32>,
}

impl ClusterReview {
    fn from_diag(cluster_diag: &ClusterDiag) -> ClusterReview {
        let mut review: ClusterReview = Default::default();
        for osd in &cluster_diag.osd_diags {
            if let Some(osd_status) = osd.osd_status.peek() {
                match *osd_status {
                    Status::NonSafe => review.not_removable.push(osd.osd_id),
                    Status::Safe => review.removable.push(osd.osd_id),
                    Status::Unknown => review.pending.push(osd.osd_id),
                }
            }
        }
        review
    }
}

#[derive(Debug, Serialize)]
pub struct ClusterDiag {
    status: Status,
    osd_diags: Vec<OsdDiag>,
}

impl ClusterDiag {
    fn new() -> ClusterDiag {
        ClusterDiag {
            status: Status::Safe,
            osd_diags: Vec::new(),
        }
    }

    fn print(&mut self, format: Format) {
        match format {
            Format::Pretty => self.print_pretty(),
            Format::Json => self.print_json(),
        };
    }

    fn status(&mut self) -> Status {
        for osd in &self.osd_diags {
            if let Some(osd_status) = osd.osd_status.peek() {
                // ClusterDiag.status defaults to safe and is only changed once
                // an OSD that is unsafe to remove or pending is found
                match *osd_status {
                    // Short circuit if we find non safe status
                    Status::NonSafe => return Status::NonSafe,
                    Status::Unknown => return Status::Unknown,
                    _ => (),
                };
            }
        }
        // Edge case where no osds are found
        if self.osd_diags.is_empty() {
            return Status::NonSafe;
        }
        self.status
    }

    fn print_pretty(&self) {
        println!("Current OSD statuses:");
        for osd in &self.osd_diags {
            if let Some(osd_status) = osd.osd_status.peek() {
                match *osd_status {
                    Status::NonSafe => println!(
                        "{} {}: {}",
                        Colour::Red.paint("●"),
                        osd.osd_id,
                        osd_status
                    ),
                    Status::Safe => println!(
                        "{} {}: {}",
                        Colour::Green.paint("●"),
                        osd.osd_id,
                        osd_status
                    ),
                    Status::Unknown => println!(
                        "{} {}: {}",
                        Colour::Yellow.paint("●"),
                        osd.osd_id,
                        osd_status
                    ),
                }
            }
        }
    }

    fn print_json(&self) {
        if let Ok(json) = serde_json::to_string(&ClusterReview::from_diag(&self)) {
            println!("{}", json);
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiagMap {
    pg_map: PGMap,
    osd_map: OsdMap,
}

impl DiagMap {
    pub fn new() -> Result<DiagMap, CSDError> {
        Ok(DiagMap {
            pg_map: PGMap::from_ceph("pg dump")?,
            osd_map: OsdMap::from_ceph("osd dump")?,
        })
    }

    // Quick check to see if `min_size +1` is satisfied
    pub fn quick_diag(self, format: Format) -> bool {
        let mut safe: bool = false;
        for stat in self.pg_map.pg_stats {
            for pool in self.osd_map.pools.iter() {
                if (stat.up.len() as i32) >= (pool.min_size + 1) {
                    safe = true;
                }
            }
        }
        match format {
            Format::Pretty => {
                if safe {
                    println!("{} Safe to remove an OSD", Colour::Green.paint("●"));
                } else {
                    println!("{} Not safe to remove an OSD", Colour::Red.paint("●"));
                };
            }
            Format::Json => println!("{{\"Safe to remove an OSD\":{}}}", safe),
        };
        safe
    }

    // Maps out PGs and their states to each OSD in their `acting` list.
    // Returns a more general `Status` based on whether there is a removable
    // OSD or not.
    // `cluster_diag` holds an OSD's removability status. Using a binary heap we
    // can always know which state it has that holds the highest precedent.
    pub fn exhaustive_diag(self, format: Format) -> Status {
        let mut pg_diags: Vec<PgDiag> = Vec::new();
        let mut cluster_diag = ClusterDiag::new();

        // Populate PG statuses. For each PG we push it's list of acting OSDs
        // and the state of the PG
        for pg_stat in self.pg_map.pg_stats {
            for acting in pg_stat.acting {
                pg_diags.push(PgDiag::new(
                    acting,
                    PgInfo::new(&pg_stat.state, pg_stat.pgid.clone()),
                ));
            }
        }

        // Generate OSD removability.
        for pg in &pg_diags {
            if cluster_diag
                .osd_diags
                .iter_mut()
                .find(|ref osd| osd.osd_id == pg.osd_id)
                .is_none()
            {
                cluster_diag.osd_diags.push(OsdDiag::new(pg.osd_id));
            } else if let Some(mut osd) = cluster_diag
                .osd_diags
                .iter_mut()
                .find(|ref osd| osd.osd_id == pg.osd_id)
            {
                match pg.pg_info.rm_safety {
                    RmSafety::None => osd.osd_status.push(Status::NonSafe),
                    RmSafety::Pending => osd.osd_status.push(Status::Unknown),
                    RmSafety::Total => osd.osd_status.push(Status::Safe),
                }
            }
        }

        // Print the statuses of OSDs based on `format`
        cluster_diag.print(format);
        cluster_diag.status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use from::FromFile;
    use osdmap::OsdMap;
    use pgmap::PGMap;

    #[test]
    fn quick_diag_jewel_safe() {
        let status = DiagMap {
            pg_map: PGMap::from_file("test/jewel/pg_dump_safe.json").unwrap(),
            osd_map: OsdMap::from_file("test/jewel/osd_dump_safe.json").unwrap(),
        }.quick_diag(Format::Pretty);

        assert_eq!(status, true);
    }

    #[test]
    fn exhaustive_diag_jewel_safe() {
        let status: Status = DiagMap {
            pg_map: PGMap::from_file("test/jewel/pg_dump_safe.json").unwrap(),
            osd_map: OsdMap::from_file("test/jewel/osd_dump_safe.json").unwrap(),
        }.exhaustive_diag(Format::Json);

        assert_eq!(status, Status::Safe);
    }

    #[test]
    fn exhaustive_diag_jewel_non_safe() {
        let status: Status = DiagMap {
            pg_map: PGMap::from_file("test/jewel/pg_dump_non_safe.json").unwrap(),
            osd_map: OsdMap::from_file("test/jewel/osd_dump_non_safe.json").unwrap(),
        }.exhaustive_diag(Format::Pretty);

        assert_eq!(status, Status::NonSafe);
    }

    #[test]
    fn exhaustive_diag_luminous_safe() {
        let status: Status = DiagMap {
            pg_map: PGMap::from_file("test/jewel/pg_dump_safe.json").unwrap(),
            osd_map: OsdMap::from_file("test/jewel/osd_dump_safe.json").unwrap(),
        }.exhaustive_diag(Format::Json);

        assert_eq!(status, Status::Safe);
    }

    #[test]
    fn exhaustive_diag_luminous_non_safe() {
        let status: Status = DiagMap {
            pg_map: PGMap::from_file("test/luminous/pg_dump_non_safe.json").unwrap(),
            osd_map: OsdMap::from_file("test/luminous/osd_dump_non_safe.json").unwrap(),
        }.exhaustive_diag(Format::Pretty);

        assert_eq!(status, Status::NonSafe);
    }

    #[test]
    fn exhaustive_diag_jewel_pending() {
        let status: Status = DiagMap {
            pg_map: PGMap::from_file("test/jewel/pg_dump_pending.json").unwrap(),
            osd_map: OsdMap::from_file("test/jewel/osd_dump_pending.json").unwrap(),
        }.exhaustive_diag(Format::Json);

        assert_eq!(status, Status::Unknown);
    }

    #[test]
    fn quick_diag_firefly_safe() {
        let status = DiagMap {
            pg_map: PGMap::from_file("test/firefly/pg_dump_safe.json").unwrap(),
            osd_map: OsdMap::from_file("test/firefly/osd_dump_safe.json").unwrap(),
        }.quick_diag(Format::Json);

        assert_eq!(status, true);
    }

    #[test]
    fn exhaustive_diag_firefly_safe() {
        let status: Status = DiagMap {
            pg_map: PGMap::from_file("test/firefly/pg_dump_safe.json").unwrap(),
            osd_map: OsdMap::from_file("test/firefly/osd_dump_safe.json").unwrap(),
        }.exhaustive_diag(Format::Pretty);

        assert_eq!(status, Status::Safe);
    }

}
