#[sidevm::main]
async fn main() {
    loop {
        sidevm::time::sleep(std::time::Duration::from_secs(10)).await
    }
}
