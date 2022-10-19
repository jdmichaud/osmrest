// Add to Cargo.toml:
// [dependencies]
// rusqlite = { version = "0.27.0", features = ["bundled", "load_extension"] }

use rusqlite::{params, Connection, LoadExtensionGuard};
use std::error::Error;
use structopt::StructOpt;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    dbfile: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
  let opt = Opt::from_args();
  // Open the file
  let conn = Connection::open(&opt.dbfile)?;
  // Load the spatialite extention
  unsafe {
    let _guard = LoadExtensionGuard::new(&conn)?;
    // If you only provide the filename, there will be automatic resolution which might work or not.
    conn.load_extension("/usr/lib/x86_64-linux-gnu/mod_spatialite.so.7.1.0", None)?;
  }

  conn.execute(
    "CREATE TABLE IF NOT EXISTS test (
      id              INTEGER PRIMARY KEY,
      name            TEXT NOT NULL,
      position        POINT
    );",
    [],
  )?;
  // Here we create a geometric point using MakePoint,
  // see: http://www.gaia-gis.it/gaia-sins/spatialite-sql-4.2.0.html
  conn.execute(
    "INSERT INTO test (id, name, position)
     VALUES (?1, ?2, MakePoint(?3, ?4))
     ON CONFLICT(id) DO UPDATE SET
      name=name,
      position=position
     ;
    ",
    params![42, "name", 1, 1],
  )?;
  // Here we create a point using GeomFromText
  let mut stmt = conn.prepare(
    "SELECT name, AsText(position) FROM test
     WHERE within(GeomFromText(?1), test.position)
     ;
    ",
  )?;
  let mut rows = stmt.query(params!["POINT(1 1)"])?;
  while let Some(row) = rows.next()? {
    println!("{} {}", row.get::<usize, String>(0)?, row.get::<usize, String>(1)?);
  }
  // within is going to return nothing here because we are far away from the position
  let mut stmt = conn.prepare(
    "SELECT name, AsText(position) FROM test
     WHERE within(GeomFromText(?1), test.position)
     ;
    ",
  )?;
  let mut rows = stmt.query(params!["POINT(2 1)"])?; // Nothing returned here
  while let Some(row) = rows.next()? {
    // Nothing displayed
    println!("{} {}", row.get::<usize, String>(0)?, row.get::<usize, String>(1)?);
  }

  Ok(())
}
