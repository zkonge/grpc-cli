use std::path::PathBuf;

use argh::FromArgs;
use http::uri::PathAndQuery;
use prost_reflect::{DescriptorPool, DynamicMessage};
use tonic::{Request, Response, client::Grpc, metadata::MetadataKey, transport::Endpoint};

use super::Executable;
use crate::{
    codec::DynamicProstCodec,
    util::{load_descriptor_set_from_path, tokio_rt},
};

/// Print the version of the application.
#[derive(FromArgs, Clone, Debug)]
#[argh(subcommand, name = "client")]
pub struct ClientCommand {
    /// the target server address, it should contain the scheme, e.g. `http://` and `unix://`
    #[argh(option, short = 's')]
    server: String,

    /// the path to the proto descriptor set file
    #[argh(option, short = 'f')]
    file_descriptor_set: PathBuf,

    /// disable package emission, which means the package name will not be used in the request
    #[argh(switch)]
    disable_package_emission: bool,

    /// the gRPC method to call, e.g. `helloworld.Greeter.SayHello`
    #[argh(positional)]
    method: String,

    /// the request data in JSON format. Leave it empty to use the default value.
    #[argh(option, short = 'd')]
    data: Option<String>,

    /// the request header in format of `key=value`. This option can be used multiple times.
    #[argh(option, short = 'h')]
    header: Vec<String>,
}

impl Executable for ClientCommand {
    fn run(&self) -> anyhow::Result<()> {
        let fdset = load_descriptor_set_from_path(&self.file_descriptor_set)?;
        let pool = DescriptorPool::from_file_descriptor_set(fdset)?;

        let (service_name, method_name) = self
            .method
            .rsplit_once(".")
            .ok_or_else(|| anyhow::anyhow!("Invalid method format. Expected `service.method`"))?;

        let service = pool
            .get_service_by_name(service_name)
            .ok_or_else(|| anyhow::anyhow!("Service not found: {}", service_name))?;

        let method = service
            .methods()
            .find(|x| x.name() == method_name)
            .ok_or_else(|| anyhow::anyhow!("Method not found: {}", method_name))?;

        let headers = self
            .header
            .iter()
            .map(|x| {
                let (key, value) = x.split_once('=').unwrap_or((x.as_str(), ""));
                (key.to_string(), value.to_string())
            })
            .collect::<Vec<_>>();

        let req_type = method.input();
        let resp_type: prost_reflect::MessageDescriptor = method.output();

        let req_msg_json = self.data.as_deref().unwrap_or("{}");
        let mut req_msg_json_de = serde_json::de::Deserializer::from_str(req_msg_json);

        let req_msg = DynamicMessage::deserialize(req_type.clone(), &mut req_msg_json_de)?;

        let resp = tokio_rt().block_on(call_grpc_method(
            self.server.to_string(),
            format!(
                "/{}/{method_name}",
                if self.disable_package_emission {
                    service.name()
                } else {
                    service.full_name()
                }
            ),
            headers,
            req_msg,
            DynamicProstCodec::new(req_type.clone(), resp_type.clone()),
        ))?;

        let resp_msg = serde_json::to_string(&resp)?;

        println!("{resp_msg}");

        Ok(())
    }
}

async fn call_grpc_method(
    server: String,
    path: String,
    headers: Vec<(String, String)>,
    msg: DynamicMessage,
    codec: DynamicProstCodec,
) -> anyhow::Result<DynamicMessage> {
    let ch = Endpoint::from_shared(server.clone())?.connect().await?;
    let mut c = Grpc::new(ch);

    let path = PathAndQuery::from_maybe_shared(path).unwrap();
    let mut req = Request::new(msg);
    for (key, value) in headers {
        req.metadata_mut().insert(
            MetadataKey::from_bytes(key.as_bytes()).unwrap(),
            value.parse().unwrap(),
        );
    }

    c.ready().await?;
    let resp: Response<DynamicMessage> = c.unary(req, path, codec).await?;

    Ok(resp.into_inner())
}
