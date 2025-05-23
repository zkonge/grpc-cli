use std::{net::SocketAddr, path::PathBuf, time::Duration};

use argh::FromArgs;
use prost_reflect::DynamicMessage;
use tonic::transport::Server;

use super::Executable;
use crate::{
    codec::DynamicProstCodec, descriptor_set::DescriptorSet, static_server::StaticService,
    util::new_tokio_rt,
};

/// acting as a server to handle a gRPC method
#[derive(FromArgs, Clone, Debug)]
#[argh(subcommand, name = "server")]
pub struct ServerCommand {
    /// the target server address, it should be a socket address, e.g. `[::]:50051`.
    #[argh(option, short = 'b')]
    bind_addr: SocketAddr,

    /// the path to the grpc proto descriptor set file. could be generated by `protoc` or `compile` command of this tool.
    #[argh(option, short = 'D')]
    descriptor_set: PathBuf,

    /// disable package emission, which means the package name will not be used in the request.
    #[argh(switch)]
    disable_package_emission: bool,

    /// the gRPC method to call, e.g. `helloworld.Greeter.SayHello`
    #[argh(positional)]
    method: String,

    /// the request data in JSON format. Leave it empty to use the default value.
    #[argh(option, short = 'd')]
    data: Option<String>,

    /// response stream cycle time, in seconds. This option is only valid for server streaming methods.
    #[argh(option)]
    stream_cycle: Option<u64>,
}
impl Executable for ServerCommand {
    fn run(&self) -> anyhow::Result<()> {
        let ds = DescriptorSet::from_file(&self.descriptor_set)?;
        let pool = ds.pool();

        let (service_name, method_name) = self.method.rsplit_once(".").ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid method format. It should look like `helloworld.Greeter.SayHello`"
            )
        })?;

        let service = pool
            .get_service_by_name(service_name)
            .ok_or_else(|| anyhow::anyhow!("Service not found: {service_name}"))?;

        let method = service
            .methods()
            .find(|x| x.name() == method_name)
            .ok_or_else(|| anyhow::anyhow!("Method not found: {method_name}"))?;

        let req_type = method.input();
        let resp_type = method.output();

        let resp_msg = match &self.data {
            Some(data) => {
                let mut resp_msg_json_de = serde_json::de::Deserializer::from_str(data);
                let msg = DynamicMessage::deserialize(resp_type.clone(), &mut resp_msg_json_de)?;
                resp_msg_json_de.end()?;
                msg
            }
            None => DynamicMessage::new(resp_type.clone()),
        };

        let svc = StaticService::new(
            // yes, this is reversed.
            DynamicProstCodec::new(resp_type, req_type),
            if self.disable_package_emission {
                service.name()
            } else {
                service.full_name()
            },
            method_name,
            method.clone(),
            resp_msg,
            self.stream_cycle.map(Duration::from_secs),
        )?;

        new_tokio_rt()
            .block_on(Server::builder().serve(self.bind_addr, svc))
            .map_err(Into::into)
    }
}
