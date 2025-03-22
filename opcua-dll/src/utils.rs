use chrono::Utc;
use libc::c_double;

const MAC_EPOCH_OFFSET: f64 = 2082844800.0; // 1904-01-01 to 1970-01-01 in seconds

//==============================================================================
// Will be used later to get TimeStaps in LabVIEW
//
#[unsafe(no_mangle)]
pub extern "C" fn get_current_cocoa_timestamp() -> c_double {
	let now = Utc::now();
	let unix_seconds = now.timestamp() as f64;
	let nanos_fraction = now.timestamp_subsec_nanos() as f64 / 1e9;

	(unix_seconds + nanos_fraction) + MAC_EPOCH_OFFSET
}
