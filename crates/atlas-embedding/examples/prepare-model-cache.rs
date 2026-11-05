use std::{error::Error, path::PathBuf};

use atlas_embedding::{
    EmbeddingRuntimeConfig, prepare_embedding_model_cache, supported_embedding_model_ids,
};

fn main() -> Result<(), Box<dyn Error>> {
    let [ref cache] = std::env::args_os().skip(1).collect::<Vec<_>>()[..] else {
        return Err("usage: prepare-model-cache CACHE_PATH".into());
    };
    let cache = PathBuf::from(cache);
    for id in supported_embedding_model_ids() {
        let config = EmbeddingRuntimeConfig::new(id.parse()?, &cache);
        let downloaded = prepare_embedding_model_cache(&config)?;
        println!(
            "{id}: verified model cache ({} files downloaded)",
            downloaded.len()
        );
    }
    Ok(())
}
