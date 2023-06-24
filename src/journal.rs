
struct JournalReader {
    entries: Vec<Entry>,
    receiver: Receiver<Entry>,
    work: JoinHandle<Result<(), Error>>,
}

