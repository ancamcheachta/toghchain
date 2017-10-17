#[macro_use]
extern crate serde_derive;

extern crate bson;
extern crate mongodb;
extern crate serde;
extern crate serde_json;

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

#[derive(Serialize, Deserialize, Debug, Default)]
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

#[derive(Serialize, Deserialize, Debug, Default)]
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
    fn get_database(&self) -> String {
        match *self {
            ElectionDatabase::Assembly(_, ref database) => database.to_string(),
            ElectionDatabase::Dail(_, ref database) => database.to_string(),
            ElectionDatabase::Westminster(_, ref database) => database.to_string(),
        }
    }
}

trait CreateDatabase {
	// Validate constituencies directory exists, then read and walk it
	fn create_constituencies(&self);
	// Validate constituencies contents are directories, then read and walk them
	fn walk_constituency_subdirs(&self, dir: &PathBuf);
	// Validate subdirectory election json files exist, then read and walk them
	fn walk_json_files(&self, subdir: &PathBuf);
	// Serialise election json with serde
	fn load_json(&self, jsonfile: &PathBuf);
	// Serialise doc as BSON and make mongodb request
	fn create_document<'a, T>(&self, doc: &'a T) where T: serde::Serialize;
}

impl CreateDatabase for ElectionDatabase {
    fn create_constituencies(&self) {
        let constituencies_dir = current_dir().unwrap().join(CONSTITUENCIES);
        
        match constituencies_dir.exists() {
            true    => self.walk_constituency_subdirs(&constituencies_dir),
            false   => {
                    eprintln!("[mongoloid] Current working directory is missing '{}' sub-directory. Exiting.", CONSTITUENCIES);
                    process::exit(1);
                },
        }
    }
    fn walk_constituency_subdirs(&self, constituencies_dir: &PathBuf) {
        let dir = fs::read_dir(constituencies_dir).unwrap();
        dir.filter_map(|entry| {
            let entry = entry.unwrap();
            let subdir: PathBuf = entry.path();
            
            match subdir.is_dir() {
                true    => Some(self.walk_json_files(&subdir)),
                false   => None
            }

        }).collect::<Vec<_>>();
    }
    fn walk_json_files(&self, subdir: &PathBuf) {
        match fs::read_dir(subdir) {
            Ok(dir) => {
                    dir.filter_map(|entry| {
                        let entry = entry.unwrap();
                        let jsonfile: PathBuf = entry.path();
                        
                        match jsonfile.is_file() {
                            true    => Some(self.load_json(&jsonfile)),
                            false   => None
                        }
                    }).collect::<Vec<_>>();
                },
            Err(why)  => panic!("[mongoloid] Couldn't read directory. {}", why),
        };
    }
    fn load_json(&self, jsonfile: &PathBuf) {
        println!("[mongoloid] Processing {:?}", jsonfile);
        match File::open(&jsonfile) {
            Err(why) => panic!("Couldn't open file: {}", why),
            Ok(file) => {
                let constituency: Area = match serde_json::from_reader(file) {
                    Err(why) => panic!("[mongoloid] Couldn't deserialise: {}", why),
                    Ok(constituency) => constituency,
                };
                self.create_document(&constituency);
            },
        };
    }
    fn create_document<'a, T>(&self, doc: &'a T) where T: serde::Serialize {
        match bson::to_bson(doc) {
            Err(why) => panic!("Couldn't convert to BSON: {}", why),
            Ok(d)    => {
                if let ElectionDatabase::Dail(ref client, ref db) = *self {
                    let coll = client.db(db).collection("area");
                    if let bson::Bson::Document(bson_doc) = d {
                        coll.insert_one(bson_doc, None)
                            .ok().expect("Failed to insert document.");
                    }
                }
            }
        }
    }
}

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
