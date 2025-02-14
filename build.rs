extern crate prost_build;

fn main() {
    // prost_build::Config::new()
    //     .type_attribute(".", "#[derive(Default)]")
    //     .compile_protos(&["src/api.proto"], &["src/"]).unwrap();
    prost_build::compile_protos(&["src/api.proto"],
                                &["src/"]).unwrap();
}
