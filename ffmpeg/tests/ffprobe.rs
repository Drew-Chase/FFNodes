mod test {
    use std::fs::read_dir;

    #[test]
    fn deserialize_probes() {
        let files = read_dir("./tests/probe_tests").unwrap();
        for file in files {
            let file = file.unwrap();
            let path = file.path();
            let content = std::fs::read_to_string(path).unwrap();
            serde_json::from_str::<ffmpeg::builders::ProbeResult>(content.as_str()).unwrap();
        }
    }
}
