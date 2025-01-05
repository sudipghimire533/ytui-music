pub mod api_info;
pub mod attachments;
pub mod channel;
pub mod common;
pub mod params;
pub mod playlists;
pub mod region;
pub mod video;

#[cfg(test)]
pub mod tests {
    

    use common::SearchResult;

    use super::*;
    

    #[test]
    fn verify_search_result_response() {
        let search_response_file = concat!(env!("CARGO_MANIFEST_DIR"), "/res/search_response.json");
        let search_response =
            std::io::BufReader::new(std::fs::File::open(search_response_file).unwrap());
        let mut serialized_search_results =
            serde_json::from_reader::<_, common::SearchResults>(search_response).unwrap();

        eprintln!("{:?}", serialized_search_results.last());
        let SearchResult::Video(last_result) = serialized_search_results.pop().unwrap() else {
            panic!("Expected video");
        };
    }
}
