use custom_logger as log;
use surrealkv::{Tree, TreeBuilder};

pub fn get_error(msg: String) -> Box<dyn std::error::Error> {
    Box::from(format!("{}", msg.to_lowercase()))
}

pub fn get_opts(db: String) -> Result<Tree, Box<dyn std::error::Error>> {
    log::debug!("[get_opts] db_path {}", db);
    let tree = TreeBuilder::new()
        .with_path(format!("{}.kv", db).into())
        .with_max_memtable_size(100 * 1024 * 1024)
        .with_block_size(4096)
        .with_level_count(1);
    let t = tree.build()?;
    log::trace!("[get_opts] tree built");
    Ok(t)
}
