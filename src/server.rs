use horrorshow::prelude::*;
use horrorshow::helper::doctype;
use tiny_http::Header;
use tiny_http::Response;
use tiny_http::Server as HTTPServer;
use url::Url;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use crate::block_tree::BlockTree;
use std::thread;
use crate::block::Block;
use crate::network::Network;

pub struct Server {
    stores: HashMap<u8, Arc<RwLock<BlockTree>>>,
    handle: HTTPServer,
}

static ERROR_404: &str = "<!DOCTYPE html>
<html>
	<body>
		<p>Content not found</p>
	</body>
</html>";

/// This macro serves the string as html
macro_rules! serve_string {
    ( $req:expr, $message:expr ) => {{
        let content_type = "Content-Type: text/html".parse::<Header>().unwrap();
        let resp = Response::from_string($message)
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}

/// This macro serves the json
macro_rules! serve_json {
    ( $req:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let resp = Response::from_string($message)
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}
// /// This macro serves the static file at the location `path` and attaches the content type `type`.
// macro_rules! serve_static_file {
//     ( $req:expr, $path:expr, $type:expr ) => {{
//         let content_type = concat!("Content-Type: ", $type).parse::<Header>().unwrap();
//         let cache_control = "Cache-Control: public, max-age=31536000"
//             .parse::<Header>()
//             .unwrap();
//         let resp = Response::from_string(include_str!($path))
//             .with_header(content_type)
//             .with_header(cache_control);
//         $req.respond(resp).unwrap();
//     }};
// }
//
// /// This macro serves the string `src` and attaches the content type `type`. Before serving the
// /// string, all occurrances of `SERVER_IP_ADDR` and `SERVER_PORT_NUMBER` in the string are replaced
// /// with the server IP and port respectively.
// macro_rules! serve_dynamic_file {
//     ( $req:expr, $src:expr, $type:expr, $addr:expr ) => {{
//         let source = $src
//             .to_string()
//             .replace("SERVER_IP_ADDR", &$addr.ip().to_string())
//             .replace("SERVER_PORT_NUMBER", &$addr.port().to_string());
//         let content_type = concat!("Content-Type: ", $type).parse::<Header>().unwrap();
//         let cache_control = "Cache-Control: no-store".parse::<Header>().unwrap();
//         let allow_all = "Access-Control-Allow-Origin: *".parse::<Header>().unwrap();
//         let resp = Response::from_string(source)
//             .with_header(content_type)
//             .with_header(cache_control)
//             .with_header(allow_all);
//         $req.respond(resp).unwrap();
//     }};
// }

impl Server {
    pub fn start(
        addr: std::net::SocketAddr,
        stores: HashMap<u8, Arc<RwLock<BlockTree>>>,
        delay: &HashMap<(u8,u8), u64>,
    ) {
        let handle = HTTPServer::http(&addr).unwrap();
        let server = Self {
            stores,
            handle,
        };
        let delay = delay.clone();
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                let stores = server.stores.clone();
                let delay = delay.clone();
                thread::spawn(move || {
                    // a valid url requires a base
                    let base_url = Url::parse(&format!("http://{}/", &addr)).unwrap();
                    let url = match base_url.join(req.url()) {
                        Ok(u) => u,
                        Err(_) => {
                            let content_type = "Content-Type: text/html".parse::<Header>().unwrap();
                            let resp = Response::from_string(ERROR_404)
                                .with_header(content_type)
                                .with_status_code(404);
                            req.respond(resp).expect("respond error");
                            return;
                        }
                    };
                    let params = url.query_pairs();
                    let params: HashMap<_, _> = params.into_owned().collect();
                    match url.path() {
                        "/dashboard" => {
                            let refresh = if let Some(_) = params.get("refresh") {
                                true
                            } else {
                                false
                            };
                            let last_number = stores.values().map(|store| {
                                let read = store.read().unwrap();
                                read.tip.number
                            }).max().expect("Error when find max tip");
                            let mut stores_for_output: HashMap<(u8,u64),HashSet<Block>> = Default::default();
                            for id in 0..stores.len() as u8 {
                                let read = stores.get(&id).unwrap();
                                let read = read.read().unwrap();
                                let tip_number = read.tip.number;
                                for level in 0..=tip_number {
                                    if let Some(blocks ) = read.number_block.get(&level) {
                                        stores_for_output.insert((id, level), blocks.clone());
                                    }
                                }
                            }
                            let page = format!("{}", html! {
                                : doctype::HTML;
                                html {
                                    head {
                                        title : "Blockchain Dashboard";
                                        style {
                                            : r".node0{color:green}.node1{color:blue}.node2{color:red}.node3{color:cyan}.node4{color:yellow}.node5{color:magenta}";
                                            : r"table, th, td { border: 1px solid black; }"
                                        }
                                        @ if refresh {
                                            meta(http-equiv="refresh", content="1");
                                        }
                                    }
                                    body {
                                        // attributes
                                        table {
                                            tr {
                                                th : "Level";
                                                @ for id in 0..stores.len() {
                                                    th(class=format_args!("node{}", id)) : format_args!("node{}", id);
                                                }
                                            }
                                            @ for level in 0..=last_number {
                                                tr {
                                                    td : format_args!("{}", level);
                                                    @ for id in 0..stores.len() as u8 {
                                                        @ if let Some(blocks) = stores_for_output.get(&(id, level)) {
                                                            td {
                                                                @ for block in blocks.iter() {
                                                                    span(class=format_args!("node{}", block.miner), title=format_args!("{}", block)) : format_args!("{} ", &hex::encode(block.digest())[..4]);
                                                                }
                                                            }
                                                        } else {
                                                            td ;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                            serve_string!(req, page)
                        }
                        "/delay" => {
                            let pretty_delay: Vec<(u8,u8,u64)> = delay.iter().map(|((i,j),k)| (*i,*j,*k)).collect();
                            serve_json!(req, serde_json::to_string_pretty(&pretty_delay).expect("Json serialize error"))
                        }
                        "/" => serve_string!(req, format!("{}", html! {
                                : doctype::HTML;
                                html {
                                    head {
                                        title : "Blockchain Dashboard";
                                    }
                                    body {
                                        p {
                                            a(href="dashboard"): "Dashboard w/o auto-refresh";
                                        }
                                        p {
                                            a(href="dashboard?refresh=1"): "Dashboard w auto-refresh";
                                        }
                                        p {
                                            a(href="delay"): "Check delay (json)";
                                        }
                                    }
                                }
                            }
                        )),
                        _ => {
                            let content_type = "Content-Type: text/html".parse::<Header>().unwrap();
                            let resp = Response::from_string(ERROR_404)
                                .with_header(content_type)
                                .with_status_code(404);
                            req.respond(resp).expect("respond error");
                        }
                    }
                });
            }
        });
    }
}