use std::collections::HashMap;
use osmpbf::{ElementReader, Element};
use std::error::Error;
use structopt::StructOpt;
use std::path::PathBuf;
use warp::Filter;
use serde::{Deserialize, Serialize};

fn file_exists(path: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path_buf = PathBuf::from(path);
    if path_buf.exists() {
        Ok(path_buf)
    } else {
        Err(format!("{} does not exists", path).into())
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, parse(try_from_str = file_exists))]
    osmfile: PathBuf,
}



#[derive(Deserialize, Serialize, Debug)]
struct Way {
  id: i64,
  tags: HashMap<String, String>,
  info: Option<InfoDef>,
  refs: Vec<i64>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Node {
  id: i64,
  tags: HashMap<String, String>,
  lat: f64,
  lon: f64,
  info: Option<InfoDef>,
}

#[derive(Deserialize, Serialize, Debug)]
struct InfoDef {
  version: Option<i32>,
  milli_timestamp: Option<i64>,
  changeset: Option<i64>,
  uid: Option<i32>,
  user: Option<String>,
  visible: bool,
  deleted: bool,
}

fn make_way(way: &osmpbf::Way) -> Way {
  Way {
    id: way.id(),
    tags: way.tags().into_iter().map(|(k, v)| (String::from(k), String::from(v))).collect(),
    refs: way.refs().into_iter().collect::<Vec<i64>>(),
    info: Some(InfoDef {
      version: way.info().version(),
      milli_timestamp: way.info().milli_timestamp(),
      changeset: way.info().changeset(),
      uid: way.info().uid(),
      user: match way.info().user() {
        Some(result) => match result {
          Ok(u) => Some(String::from(u)),
          Err(_) => None,
        },
        None => None,
      },
      visible: way.info().visible(),
      deleted: way.info().deleted(),
    }),
  }
}


fn make_node(node: &osmpbf::Node) -> Node {
  Node {
    id: node.id(),
    tags: node.tags().into_iter().map(|(k, v)| (String::from(k), String::from(v))).collect(),
    lat: node.lat(),
    lon: node.lon(),
    info: Some(InfoDef {
      version: node.info().version(),
      milli_timestamp: node.info().milli_timestamp(),
      changeset: node.info().changeset(),
      uid: node.info().uid(),
      user: match node.info().user() {
        Some(result) => match result {
          Ok(u) => Some(String::from(u)),
          Err(_) => None,
        },
        None => None,
      },
      visible: node.info().visible(),
      deleted: node.info().deleted(),
    }),
  }
}

fn make_node_from_dense_node(node: &osmpbf::DenseNode) -> Node {
  Node {
    id: node.id(),
    tags: node.tags().into_iter().map(|(k, v)| (String::from(k), String::from(v))).collect(),
    lat: node.lat(),
    lon: node.lon(),
    info: None,
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  // GET /
  let root = warp::get()
    .and(warp::path::end())
    .map(|| "OSM Rest Server");

  let log = warp::log::custom(|info| {
    // Use a log macro, or slog, or println, or whatever!
    eprintln!(
      "{} {} {}",
      info.method(),
      info.path(),
      info.status(),
    );
  });

  let nodes = warp::get()
    .and(warp::path("v1"))
    .and(warp::path("nodes"))
    .and(warp::path::end())
    .map(|| {
      let opt = Opt::from_args();
      let reader = ElementReader::from_path(&opt.osmfile).unwrap();
      let mut nodes = vec![];
      reader.for_each(|element| {
        match element {
          Element::Node(node) => {
            nodes.push(make_node(&node));
          }
          Element::DenseNode(node) => {
            nodes.push(make_node_from_dense_node(&node));
          }
          _ => (),
        }
      }).unwrap();
      serde_json::to_string(&nodes).unwrap()
    });

  let ways = warp::get()
    .and(warp::path("v1"))
    .and(warp::path("ways"))
    .and(warp::path::end())
    .map(|| {
      let opt = Opt::from_args();
      let reader = ElementReader::from_path(&opt.osmfile).unwrap();
      let mut ways = vec![];
      reader.for_each(|element| {
        match element {
          Element::Way(way) => {
            ways.push(make_way(&way));
          }
          _ => (),
        }
      }).unwrap();
      serde_json::to_string(&ways).unwrap()
    });

  let routes = root
    .or(nodes)
    .or(ways)
    .with(log)
  ;

  let address = [127, 0, 0, 1];
  let port = 8080;
  let printable_address = address.iter().map(|c: &u8| c.to_string()).collect::<Vec<String>>().join(":");

  println!("Serving HTTP on {} port {} (http://{}:{}/) ...", printable_address, port, printable_address, port);
  warp::serve(routes)
    .run((address, port))
    .await;

  Ok(())
}
