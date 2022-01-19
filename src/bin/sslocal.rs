use shadowsocks::args::parse_args;
use shadowsocks::local::Server;

fn main() {
    env_logger::init();
    let configs = parse_args("sslocal").unwrap();
    log::info!("{}", serde_json::to_string_pretty(&configs).unwrap());

    let server = Server::new(configs);
    awak::block_on(async {
        server.serve().await;
    })
}
