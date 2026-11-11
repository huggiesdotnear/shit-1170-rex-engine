// ========================================
// SERVER
// how the app is bound and how it says hello on startup.
//
// --- WHY --- these live together: the host, the port, the list of lan
// addresses and the banner that prints them are one startup concern.
// the banner reads the other two, so they belong next to it.
// ========================================
pub mod shit_1170_banner_fun;
pub mod shit_1170_host_fun;
pub mod shit_1170_local_ips_fun;
pub mod shit_1170_port_fun;
pub mod shit_1170_rpc_url_fun;
// ========================================
// copyright 2026 by sleet.near
// ========================================
