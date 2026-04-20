//! Scheduling conflicts: shared court, Basketball Full vs halves (1/2) and vs Pickleball A/B/C,
//! and BB court 1↔Pickleball C, BB court 2↔Pickleball B.

/// Half-open intervals [start, end) overlap.
pub fn time_ranges_overlap(s1: i64, e1: i64, s2: i64, e2: i64) -> bool {
    s1 < e2 && s2 < e1
}

/// True if the proposed booking conflicts with an existing one (same day, overlapping time).
pub fn scheduling_conflict(
    p_date: &str,
    p_start: i64,
    p_end: i64,
    p_sport: &str,
    p_court: &str,
    e_id: &str,
    e_date: &str,
    e_start: i64,
    e_end: i64,
    e_sport: &str,
    e_court: &str,
    exclude_existing_id: Option<&str>,
) -> bool {
    if exclude_existing_id.is_some_and(|id| id == e_id) {
        return false;
    }
    if p_date != e_date {
        return false;
    }
    if !time_ranges_overlap(p_start, p_end, e_start, e_end) {
        return false;
    }
    if p_court == e_court {
        return true;
    }
    basketball_full_vs_halves(p_sport, p_court, e_sport, e_court)
        || full_court_vs_pickleball(p_sport, p_court, e_sport, e_court)
        || bb_court_vs_pickleball_adjacent(p_sport, p_court, e_sport, e_court)
}

/// Basketball Full uses the same floor as courts 1 and 2 combined.
fn basketball_full_vs_halves(p_sport: &str, p_court: &str, e_sport: &str, e_court: &str) -> bool {
    let p_full = p_sport == "Basketball" && p_court == "Full";
    let e_full = e_sport == "Basketball" && e_court == "Full";
    let p_half = p_sport == "Basketball" && matches!(p_court, "1" | "2");
    let e_half = e_sport == "Basketball" && matches!(e_court, "1" | "2");
    (p_full && e_half) || (e_full && p_half)
}

fn full_court_vs_pickleball(p_sport: &str, p_court: &str, e_sport: &str, e_court: &str) -> bool {
    let p_full = p_sport == "Basketball" && p_court == "Full";
    let e_full = e_sport == "Basketball" && e_court == "Full";
    let p_pb = p_sport == "Pickleball" && matches!(p_court, "A" | "B" | "C");
    let e_pb = e_sport == "Pickleball" && matches!(e_court, "A" | "B" | "C");
    (p_full && e_pb) || (e_full && p_pb)
}

/// Pickleball C ↔ Basketball 1; Pickleball B ↔ Basketball 2 (shared space).
fn bb_court_vs_pickleball_adjacent(p_sport: &str, p_court: &str, e_sport: &str, e_court: &str) -> bool {
    let p_bb1 = p_sport == "Basketball" && p_court == "1";
    let e_bb1 = e_sport == "Basketball" && e_court == "1";
    let p_pbc = p_sport == "Pickleball" && p_court == "C";
    let e_pbc = e_sport == "Pickleball" && e_court == "C";
    let p_bb2 = p_sport == "Basketball" && p_court == "2";
    let e_bb2 = e_sport == "Basketball" && e_court == "2";
    let p_pbb = p_sport == "Pickleball" && p_court == "B";
    let e_pbb = e_sport == "Pickleball" && e_court == "B";
    (p_bb1 && e_pbc) || (e_bb1 && p_pbc) || (p_bb2 && e_pbb) || (e_bb2 && p_pbb)
}

pub fn validate_sport_and_court(sport: &str, court: &str) -> Result<(), String> {
    match sport {
        "Basketball" => match court {
            "1" | "2" | "Full" => Ok(()),
            _ => Err(format!(
                "Invalid court {court:?} for Basketball (expected 1, 2, or Full)"
            )),
        },
        "Pickleball" => match court {
            "A" | "B" | "C" => Ok(()),
            _ => Err(format!(
                "Invalid court {court:?} for Pickleball (expected A, B, or C)"
            )),
        },
        "Table Tennis" => {
            if court == "Table Tennis" {
                Ok(())
            } else {
                Err(format!(
                    "Invalid court {court:?} for Table Tennis (expected \"Table Tennis\")"
                ))
            }
        }
        _ => Err(format!(
            "Invalid sport {sport:?} (expected Basketball, Pickleball, or Table Tennis)"
        )),
    }
}
