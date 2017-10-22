//! # mongoloid
//! 
//! The `mongoloid` crate exposes a basic API with which to build a Toghcháin
//! Éireann database in MongoDB.  This is called through `main.rs` when the
//! program is executed.  `mongoloid` will:
//! 
//! - Infer the type of database to build from the current directory name (`dail`, `assembly`, or `westminster`)
//! - Find the `constituency` directory and walk all its subdirectories
//! - For each direcory, walk all its `.json` files and serialise them as native structs in Rust
//! - Insert all data into the new database

#[macro_use]
extern crate serde_derive;

extern crate bson;
extern crate mongodb;
extern crate serde;
extern crate serde_json;

use bson::ordered::OrderedDocument;
use mongodb::{Client, ClientInner, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use std::env::current_dir;
use std::fs;
use std::fs::{File};
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

pub mod util;

const ASSEMBLY: &'static str = "assembly";
const DAIL: &'static str = "dail";
const WESTMINSTER: &'static str = "westminster";
const CONSTITUENCIES: &'static str = "constituencies";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
struct Area {
    area_type: String,
    candidates: Vec<Candidate>,
    counts_held: Option<i8>, 
    description: String, 
    election_type: String, 
    electorate: Option<i32>, 
    name: String, 
    quota: Option<i32>, 
    spoilt: Option<i16>, 
    turnout: Option<i32>, 
    valid: Option<i32>, 
    year: i16,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
struct Candidate {
    counts: Vec<i32>,
    elected: bool,
    first_pref_pc: Option<f32>,
    full_name: String,
    party: String,
    transfers: Option<i32>,
    transfers_pc: Option<f32>,
}

pub enum ElectionDatabase {
    Assembly(Arc<ClientInner>, String),
    Dail(Arc<ClientInner>, String),
    Westminster(Arc<ClientInner>, String),
}

impl ElectionDatabase {
    /// Creates an `ElectionDatabase` enum from the `input` value provided.  An
    /// `Option` will be returned wrapping the appropriate variant as matched
    /// with the relevant constant `&'static str` representing election types
    /// currently supported by `mongoloid`.  As of 0.1.0, these are the
    /// constants:
    /// * `ASSEMBLY`
    /// * `DAIL`
    /// * `WESTMINSTER`
    /// 
    /// As for `db_name`, where the `Option` variant is `Some`, the wrapped
    /// value will be moved and wrapped within the new `ElectionDatabase` 
    /// variant as the `db` field value.  Where the `Option` variant is `None`,
    /// a unique name will be generated to avoid naming conflicts (eg. 
    /// `"dail_f404b"`) and wrapped within the new `ElectionDatabase` variant
    /// as the `db` field value.  The `db` field value will later be used as the
    /// MongoDB database name once the data is inserted.
    ///
    /// Finally, the variant returned will also wrap a `client` value which is
    /// obtained durint the execution of this method.  This client is created
    /// from the MongoDB URI `"mongodb://127.0.0.1:27017"`, which is the only
    /// URI supported at present.
    fn from_str(input: &str, db_name: Option<&str>) -> Option<Self> {
        let mut db = String::new();
        
        match db_name {
            Some(s) => db.clone_from(&s.to_string()),
            None    => {
                let unique = util::mini_hash();
                db.clone_from(&format!("{}_{}", input, unique));
            }
        };
        
        let client = Client::with_uri("mongodb://127.0.0.1:27017")
            .expect("Failed to initialize standalone client.");
        
        match input.as_ref() {
            ASSEMBLY    => Some(ElectionDatabase::Assembly(client, db)),
            DAIL        => Some(ElectionDatabase::Dail(client, db)),
            WESTMINSTER => Some(ElectionDatabase::Westminster(client, db)),
            _           => None,
        }
    }
    /// References the `db` value wrapped in the current `ElectionDatabase`
    /// variant and returns its value as a new `String`.
    fn get_database(&self) -> String {
        match *self {
            ElectionDatabase::Assembly(_, ref database) => database.to_string(),
            ElectionDatabase::Dail(_, ref database) => database.to_string(),
            ElectionDatabase::Westminster(_, ref database) => database.to_string(),
        }
    }
}

trait CreateDatabase {
	/// Validate constituencies directory exists, then read and walk it.
	/// Create constituencies from all the data read.
	fn create_constituencies(&self);
	/// Validate constituencies contents are directories, then read and walk
	/// them.  Return an `Option<Vec<Area>>` from the data read.
	fn walk_constituency_subdirs(&self, dir: &PathBuf) -> Option<Vec<Area>>;
	/// Validate subdirectory election `.json` files exist, then read and walk
	/// them. Return an `Option<Vec<Area>>` from the data read.
	fn walk_json_files(&self, subdir: &PathBuf) -> Option<Vec<Area>>;
	/// Serialise an election `.json` file with serde.  Return an `Option<Area>`
	/// from the data read.
	fn load_json(&self, jsonfile: &PathBuf) -> Option<Area>;
	/// Serialise `areas` as a `Vec<OrderedDocument` and insert to MongoDB.
	/// **Note:** collection name will always be `area`.
	fn create_documents<'a,T>(&self, areas: &'a Vec<T>) where T: serde::Serialize;
}

impl CreateDatabase for ElectionDatabase {
    fn create_constituencies(&self) {
        let constituencies_dir = current_dir().unwrap().join(CONSTITUENCIES);
        
        match constituencies_dir.exists() {
            true    => match self.walk_constituency_subdirs(&constituencies_dir) {
                    Some(ref a) => self.create_documents(a),
                    None        => {
                        eprintln!("{}","[mongoloid] No constituencies to create.");
                        process::exit(1);
                    }
                },
            false   => {
                    eprintln!("[mongoloid] Current working directory is missing '{}' sub-directory. Exiting.", CONSTITUENCIES);
                    process::exit(1);
                },
        }
    }
    fn walk_constituency_subdirs(&self, constituencies_dir: &PathBuf) -> Option<Vec<Area>> {
        let dir = fs::read_dir(constituencies_dir).unwrap();
        let areas: Vec<Area> = dir.filter_map(|entry| {
            let entry = entry.unwrap();
            let subdir: PathBuf = entry.path();
            
            match subdir.is_dir() {
                true    => self.walk_json_files(&subdir),
                false   => None
            }

        }).collect::<Vec<Vec<Area>>>()
        .iter()
        .flat_map(|a| a.iter().map(Clone::clone))
        .collect();
        
        match areas.is_empty() {
            true    => None,
            false   => Some(areas)
        }
    }
    fn walk_json_files(&self, subdir: &PathBuf) -> Option<Vec<Area>> {
        let mut areas: Vec<Area> = Vec::new();
        match fs::read_dir(subdir) {
            Err(why)  => panic!("[mongoloid] Couldn't read directory. {}", why),
            Ok(dir) => {
                let a = dir.filter_map(|entry| {
                    let entry = entry.unwrap();
                    let jsonfile: PathBuf = entry.path();
                    
                    match jsonfile.is_file() {
                        true    => self.load_json(&jsonfile),
                        false   => None
                    }
                }).collect::<Vec<Area>>();
                if !a.is_empty() {
                    areas.extend(a);
                }
            },
        };
        match areas.is_empty() {
            true    => None,
            false   => Some(areas)
        }
    }
    fn load_json(&self, jsonfile: &PathBuf) -> Option<Area>{
        println!("[mongoloid] Processing {:?}", jsonfile);
        match File::open(&jsonfile) {
            Err(why) => panic!("Couldn't open file: {}", why),
            Ok(file) => {
                let area: Option<Area> = match serde_json::from_reader(file) {
                    Ok(a) => Some(a),
                    Err(why) => panic!("[mongoloid] Couldn't deserialise: {}", why),
                };
                return area;
            },
        }
    }
    fn create_documents<'a,T>(&self, areas: &'a Vec<T>) where T: serde::Serialize {
        println!("[mongoloid] Creating database \"{}\"...", self.get_database());
        let docs: Vec<OrderedDocument> = areas.iter().filter_map(|a| {
            match bson::to_bson(a) {
                Err(why) => {
                        eprintln!("[mongoloid] Couldn't convert row to BSON: {}", why);
                        None
                    },
                Ok(d) => {
                    if let bson::Bson::Document(bson_doc) = d {
                        return Some(bson_doc);
                    }
                    None
                },
            }
        }).collect();
        if let ElectionDatabase::Dail(ref client, ref db) = *self {
            let coll = client.db(db).collection("area");
            
            coll.insert_many(docs, None)
                .ok().expect("Failed to insert document.");
        }
    }
}

/// Create a new database in MongoDB.  If `db_name` contains a value, it will be
/// be used to name the database, otherwise an unique name will be automatically
/// generated.
/// **Note:** the current directory name must be that of a supported election.
pub fn create_database(db_name: Option<&str>) -> Result<(),String> {
    match ElectionDatabase::from_str(&util::get_cwd_name(), db_name){
        Some(et) => {
                println!("[mongoloid] {}", "Creating database");
                et.create_constituencies();
                println!("[mongoloid] Created database \"{}\"", et.get_database());
            },
        None     => {
                eprintln!("[mongoloid] Current working directory {:?} is not an expected election type. Exiting.", current_dir().unwrap());
                process::exit(1);
            },
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate bson;
    extern crate mongodb;

    use bson::{Bson, Document};
    use mongodb::{Client, ThreadedClient};
    use mongodb::db::ThreadedDatabase;
    use std::env::set_current_dir;
    
    #[test]
    #[allow(unused_must_use)]
    fn database_builds() {
        let test_dir = super::current_dir().unwrap().join("test");
        let dail_dir = test_dir.join("dail");
        let client = Client::with_uri("mongodb://127.0.0.1:27017")
            .expect("Failed to initialize standalone client.");
        let coll = client.db("test").collection("area");
        
        set_current_dir(&dail_dir);
        
        // Delete test area collection if it exists
        coll.delete_many(Document::new(), None);
        
        // Run our code
        super::create_database(Some("test"));
        
        // Query and assert
        let result = coll.find_one(None, None).expect(
            "Failed to execute find command.",
        );
        match result.unwrap().get("valid") {
            Some(&Bson::I32(valid)) => assert_eq!(44034 as i32, valid),
            _ => panic!("Expected Bson::I32!"),
        };
        
        // Delete test area collection again
        coll.delete_many(Document::new(), None);
    }
}
