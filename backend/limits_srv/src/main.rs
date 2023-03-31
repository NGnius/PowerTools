use std::sync::atomic::{Ordering, AtomicU64, AtomicI64};
use std::sync::{RwLock, Arc};

use serde::Serialize;
use warp::Filter;

use limits_core::json::Base;

static VISIT_COUNT: AtomicU64 = AtomicU64::new(0);

static START_TIME: AtomicI64 = AtomicI64::new(0);

fn get_limits(base: Base) -> impl warp::Reply {
    VISIT_COUNT.fetch_add(1, Ordering::AcqRel);
    //println!("Limits got");
    warp::reply::json(&base)
}

#[derive(Serialize)]
struct Visits {
    visits: u64,
    since: i64, // Unix time (since epoch)
}

fn get_visits() -> impl warp::Reply {
    let count = VISIT_COUNT.load(Ordering::Relaxed);
    let start = START_TIME.load(Ordering::Relaxed);
    //println!("Count got");
    warp::reply::json(&Visits {
        visits: count,
        since: start,
    })
}

#[allow(opaque_hidden_inferred_bound)]
fn routes(base: Arc<RwLock<Base>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(
        warp::path!("powertools" / "v1")
            .map(move || {
                let base = base.read().expect("Failed to acquire base limits read lock").clone();
                get_limits(base)
            })
        .or(
            warp::path!("powertools" / "count")
                .map(get_visits)
        )
    ).recover(recovery)
}

pub async fn recovery(reject: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if reject.is_not_found() {
        Ok(warp::hyper::StatusCode::NOT_FOUND)
    } else {
        Err(reject)
    }
}

#[tokio::main]
async fn main() {
    START_TIME.store(chrono::Utc::now().timestamp(), Ordering::Relaxed);
    let file = std::fs::File::open("./pt_limits.json").expect("Failed to read limits file");
    let limits: Base = serde_json::from_reader(file).expect("Failed to parse limits file");
    assert!(limits.refresh.is_some(), "`refresh` cannot be null, since it will brick future refreshes");

    warp::serve(routes(Arc::new(RwLock::new(limits))))
        .run(([0, 0, 0, 0], 8080))
        .await;
}

#[cfg(test)]
mod test {
    #[test]
    fn generate_default_pt_limits() {
        let limits = limits_core::json::Base::default();
        let output_file = std::fs::File::create("./pt_limits.json").unwrap();
        serde_json::to_writer_pretty(output_file, &limits).unwrap();
    }
}
