macro_rules! jsonrpc_client {
    (
        $(#[$struct_attr:meta])*
        pub struct $struct_name:ident {$(
            $(#[$attr:meta])*
            pub fn $method:ident(&self$(, $arg_name:ident: $arg_ty:ty)*) -> Result<$return_ty:ty>;
        )*}
    ) => {
        use failure::Error;
        use reqwest as rq;
        use uuid::Uuid as req_id;

        #[derive(Deserialize)]
        struct RpcResponse<T> {
            pub result: Option<T>,
            pub error: serde_json::Value,
            pub id: req_id,
        }

        #[derive(Serialize)]
        struct RpcRequest<T> {
            pub method: String,
            pub params: T,
            pub id: req_id,
        }

        $(#[$struct_attr])*
        pub struct $struct_name {
            client: rq::Client,
            uri: String,
            user: Option<String>,
            pass: Option<String>,
        }

        impl $struct_name {
            pub fn new(uri: String, user: Option<String>, pass: Option<String>) -> Self {
                $struct_name {
                    client: rq::Client::new(),
                    uri,
                    user,
                    pass,
                }
            }

            $(
                $(#[$attr])*
                pub fn $method(&self$(, $arg_name: $arg_ty)*) -> Result<$return_ty, Error> {
                    let mut builder = self.client.post(&self.uri);
                    match (&self.user, &self.pass) {
                        (Some(ref u), Some(ref p)) => builder = builder.basic_auth(u, Some(p)),
                        (Some(ref u), None) => builder = builder.basic_auth::<&str, &str>(u, None),
                        _ => (),
                    };
                    builder = builder.json(&RpcRequest {
                        method: stringify!($method).to_owned(),
                        params: ($($arg_name,)*),
                        id: req_id::new_v4(),
                    });
                    let mut res = builder.send()?;
                    let body: RpcResponse<$return_ty> = res.json()?;
                    match body.result {
                        Some(a) => Ok(a),
                        None => bail!("{:?}", body.error)
                    }
                }
            )*
        }
    };
}