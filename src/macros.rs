macro_rules! jsonrpc_client {
    (
        $(#[$struct_attr:meta])*
        pub struct $struct_name:ident {
            $(
                single:
                $(
                    $(#[$attr_a:meta])*
                    pub fn $method_a:ident(&self$(, $arg_name_a:ident: $arg_ty_a:ty)*) -> Result<$return_ty_a:ty>;
                )*
                enum:
                $(
                    $(#[$attr_b:meta])*
                    pub fn $method_b:ident(&self$(, $arg_name_b:ident: $arg_ty_b:ty)*) -> Result<$($title:ident($return_ty_b:ty))|*>;
                )*
            )+
        }
    ) => {
        use std::marker::PhantomData;
        use failure::Error;
        use reqwest as rq;
        use serde::Deserialize;
        use serde::Serialize;
        use std::sync::Mutex;
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

        impl<T> RpcRequest<T>
        where T: Serialize {
            pub fn polymorphize(self) -> RpcRequest<serde_json::Value> {
                RpcRequest {
                    method: self.method,
                    params: serde_json::from_str(&serde_json::to_string(&self.params).unwrap()).unwrap(),
                    id: self.id,
                }
            }
        }

        pub mod reply {
            use failure::Error;
            use super::*;
            $(
                $(
                    $(#[$attr_b])*
                    #[derive(Debug)]
                    #[allow(non_camel_case_types)]
                    pub enum $method_b {
                        $($title($return_ty_b),)+
                    }

                    $(#[$attr_b])*
                    impl $method_b {
                        $(
                            #[allow(non_snake_case)]
                            pub fn $title(self) -> Result<$return_ty_b, Error> {
                                match self {
                                    $method_b::$title(a) => Ok(a),
                                    a => bail!("Wrong variant of {}: expected {}(_), got {:?}", stringify!(reply::$method_b), stringify!(reply::$method_b::$return_ty_b), a)
                                }
                            }
                        )+
                    }
               )*
            )+
        }

        pub struct ReqBatcher<$struct_name> {
            reqs: Vec<RpcRequest<serde_json::Value>>,
            phantom: PhantomData<$struct_name>,
        }

        pub trait BatchRequest<$struct_name> {
            fn new() -> Self;

            $(
                $(
                    $(#[$attr_a])*
                    fn $method_a(&self$(, $arg_name_a: $arg_ty_a)*) -> req_id;
                )*
                $(
                    $(#[$attr_b])*
                    fn $method_b(&self$(, $arg_name_b: $arg_ty_b)*) -> req_id;
                )*
            )*
        }

        impl BatchRequest<$struct_name> for Mutex<ReqBatcher<$struct_name>> {
            fn new() -> Self {
                Mutex::new(ReqBatcher {
                    reqs: Vec::new(),
                    phantom: PhantomData,
                })
            }
            $(
                $(
                    $(#[$attr_a])*
                    fn $method_a(&self$(, $arg_name_a: $arg_ty_a)*) -> req_id {
                        let id = req_id::new_v4();
                        let body = RpcRequest {
                            method: stringify!($method_a).to_owned(),
                            params: ($($arg_name_a,)*),
                            id,
                        }.polymorphize();
                        self.lock().unwrap().reqs.push(body);
                        id
                    }
                )*
                $(
                    $(#[$attr_b])*
                    fn $method_b(&self$(, $arg_name_b: $arg_ty_b)*) -> req_id {
                        let id = req_id::new_v4();
                        let body = RpcRequest {
                            method: stringify!($method_b).to_owned(),
                            params: ($($arg_name_b,)*),
                            id,
                        }.polymorphize();
                        self.lock().unwrap().reqs.push(body);
                        id
                    }
                )*
            )*
        }

        $(#[$struct_attr])*
        pub struct $struct_name {
            client: rq::Client,
            uri: String,
            user: Option<String>,
            pass: Option<String>,
            mutex: bool,
            pub batcher: Mutex<ReqBatcher<$struct_name>>,
        }

        impl $struct_name {
            pub fn new(uri: String, user: Option<String>, pass: Option<String>, mutex: bool) -> Self {
                use BatchRequest;
                $struct_name {
                    client: rq::Client::new(),
                    uri,
                    user,
                    pass,
                    mutex,
                    batcher: BatchRequest::new(),
                }
            }
            $(
                $(
                    $(#[$attr_a])*
                    pub fn $method_a(&self$(, $arg_name_a: $arg_ty_a)*) -> Result<$return_ty_a, Error> {
                        let mut builder = self.client.post(&self.uri);
                        match (&self.user, &self.pass) {
                            (Some(ref u), Some(ref p)) => builder = builder.basic_auth(u, Some(p)),
                            (Some(ref u), None) => builder = builder.basic_auth::<&str, &str>(u, None),
                            _ => (),
                        };
                        builder = builder.json(&RpcRequest {
                            method: stringify!($method_a).to_owned(),
                            params: ($($arg_name_a,)*),
                            id: req_id::new_v4(),
                        });
                        let mut res = match self.mutex {
                            true => {
                                let lock = self.batcher.lock().unwrap();
                                let res = builder.send()?;
                                drop(lock);
                                res
                            },
                            false => builder.send()?
                        };
                        let txt = res.text()?;
                        let body: RpcResponse<$return_ty_a> = serde_json::from_str(&txt)?;
                        match body.result {
                            Some(a) => Ok(a),
                            None => bail!("{:?}", body.error)
                        }
                    }
                )*
                $(
                    $(#[$attr_b])*
                    pub fn $method_b(&self$(, $arg_name_b: $arg_ty_b)*) -> Result<reply::$method_b, Error> {
                        let mut builder = self.client.post(&self.uri);
                        match (&self.user, &self.pass) {
                            (Some(ref u), Some(ref p)) => builder = builder.basic_auth(u, Some(p)),
                            (Some(ref u), None) => builder = builder.basic_auth::<&str, &str>(u, None),
                            _ => (),
                        };
                        builder = builder.json(&RpcRequest {
                            method: stringify!($method_b).to_owned(),
                            params: ($($arg_name_b,)*),
                            id: req_id::new_v4(),
                        });
                        let mut res = match self.mutex {
                            true => {
                                let lock = self.batcher.lock().unwrap();
                                let res = builder.send()?;
                                drop(lock);
                                res
                            },
                            false => builder.send()?
                        };
                        let txt = res.text()?;
                        let body: reply::$method_b = (|txt: String| {
                            $(
                                match serde_json::from_str::<RpcResponse<$return_ty_b>>(&txt) {
                                    Ok(a) => match a.result {
                                        Some(b) => return Ok(reply::$method_b::$title(b)),
                                        _ => bail!("{:?}", a.error),
                                    },
                                    Err(_) => (),
                                };
                            )+
                            Err(format_err!("Cannot deserialize to any variant of reply::{}", stringify!($method_b)))
                        })(txt)?;
                        Ok(body)
                    }
                )*
            )*
            pub fn send_batch<T: for<'de> Deserialize<'de>>(&self) -> Result<HashMap<req_id, T>, Error> {
                let mut builder = self.client.post(&self.uri);
                match (&self.user, &self.pass) {
                    (Some(ref u), Some(ref p)) => builder = builder.basic_auth(u, Some(p)),
                    (Some(ref u), None) => builder = builder.basic_auth::<&str, &str>(u, None),
                    _ => (),
                };
                let mut batcher_lock = self.batcher.lock().unwrap();
                builder = builder.json(&batcher_lock.reqs);
                let mut res = builder.send()?;
                batcher_lock.reqs = Vec::new();
                drop(batcher_lock);
                let json: Vec<RpcResponse<T>> = res.json()?;
                json.into_iter().map(|reply| {
                    Ok((reply.id, match reply.result {
                        Some(b) => b,
                        _ => bail!("{:?}", reply.error),
                    }))
                }).collect()
            }
        }
    };
}