extern crate argparse;
extern crate mongoloid;

use argparse::{ArgumentParser, Store};

#[allow(unused_must_use)]
fn main() {
    let mut database = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Mongoloid election database builder");
        ap.refer(&mut database)
            .add_option(&["-d", "--database"], Store,
            "name of database to build (optional)");
        ap.parse_args_or_exit();
    }
    match database.len() > 0 {
        true    => mongoloid::create_database(Some(&database)),
        false   => mongoloid::create_database(None),
    };
}