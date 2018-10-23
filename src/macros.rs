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
        use failure::Error;
        use reqwest as rq;
        use serde::Deserialize;
        use std::marker::PhantomData;
        use serde::Serialize;
        use std::sync::{Arc, Condvar, Mutex};
        use uuid::Uuid as req_id;

        #[derive(Deserialize)]
        struct RpcResponse<T> {
            pub result: Option<T>,
            pub error: serde_json::Value,
            pub id: Option<req_id>,
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

        pub struct ReqBatcher<T> {
            reqs: Vec<RpcRequest<serde_json::Value>>,
            resps: HashMap<req_id, serde_json::Value>,
            max_batch_size: usize,
            phantom: PhantomData<T>,
        }

        pub trait BatchRequest<$struct_name> {
            fn inner(&mut self) -> &mut ReqBatcher<$struct_name>;
            $(
                $(
                    $(#[$attr_a])*
                    fn $method_a(&mut self$(, $arg_name_a: $arg_ty_a)*) -> Result<req_id, Error>;
                )*
                $(
                    $(#[$attr_b])*
                    fn $method_b(&mut self$(, $arg_name_b: $arg_ty_b)*) -> Result<req_id, Error>;
                )*
            )*
            fn flush(&mut self) -> Result<(), Error>;
            fn send<T: for<'de> Deserialize<'de>>(&mut self) -> Result<HashMap<req_id, T>, Error>;
        }

        impl<'a> BatchRequest<$struct_name> for (&'a $struct_name, ReqBatcher<$struct_name>) {
            fn inner(&mut self) -> &mut ReqBatcher<$struct_name> {
                &mut self.1
            }

            $(
                $(
                    $(#[$attr_a])*
                    fn $method_a(&mut self$(, $arg_name_a: $arg_ty_a)*) -> Result<req_id, Error> {
                        let id = req_id::new_v4();
                        let body = RpcRequest {
                            method: stringify!($method_a).to_owned(),
                            params: ($($arg_name_a,)*),
                            id,
                        }.polymorphize();
                        if self.inner().reqs.len() >= self.inner().max_batch_size {
                            self.flush()?;
                        }
                        self.inner().reqs.push(body);
                        Ok(id)
                    }
                )*
                $(
                    $(#[$attr_b])*
                    fn $method_b(&mut self$(, $arg_name_b: $arg_ty_b)*) -> Result<req_id, Error> {
                        let id = req_id::new_v4();
                        let body = RpcRequest {
                            method: stringify!($method_b).to_owned(),
                            params: ($($arg_name_b,)*),
                            id,
                        }.polymorphize();
                        if self.inner().reqs.len() >= self.inner().max_batch_size {
                            self.flush()?;
                        }
                        self.inner().reqs.push(body);
                        Ok(id)
                    }
                )*
            )*

            fn flush(&mut self) -> Result<(), Error> {
                if self.inner().reqs.len() == 0 {
                    return Ok(())
                }
                //println!("send_batch attempt");
                let mut builder = rq::Client::new()
                    .post(&self.0.uri)
                    .header(rq::header::CONNECTION, rq::header::HeaderValue::from_static("close"));
                match (&self.0.user, &self.0.pass) {
                    (Some(ref u), Some(ref p)) => builder = builder.basic_auth(u, Some(p)),
                    (Some(ref u), None) => builder = builder.basic_auth::<&str, &str>(u, None),
                    _ => (),
                };
                builder = builder.json(&self.inner().reqs);
                if self.0.rps > 0 {
                    let wait = std::time::Duration::from_secs(1) / self.0.rps as u32;
                    let mut lock = self.0.last_req.lock().unwrap();
                    let elapsed = lock.elapsed();
                    if elapsed < wait {
                        std::thread::sleep(wait - elapsed);
                    }
                    *lock = std::time::Instant::now();
                    drop(lock);
                }
                if self.0.max_concurrency > 0 {
                    let mut lock = self.0.counter.0.lock().unwrap();
                    while *lock == self.0.max_concurrency {
                        lock = self.0.counter.1.wait(lock).unwrap();
                    }
                    if *lock > self.0.max_concurrency {
                        unreachable!();
                    }
                    *lock = *lock + 1;
                    drop(lock);
                }
                #[cfg(feature = "logging")] println!("batch {}", self.inner().reqs.len());
                let mut res = match builder.send() {
                    Ok(a) => a,
                    Err(e) => {
                        if self.0.max_concurrency > 0 {
                            let mut lock = self.0.counter.0.lock().unwrap();
                            *lock = *lock - 1;
                            drop(lock);
                            self.0.counter.1.notify_one();
                        }
                        Err(e)?
                    }
                };
                if self.0.max_concurrency > 0 {
                    let mut lock = self.0.counter.0.lock().unwrap();
                    *lock = *lock - 1;
                    drop(lock);
                    self.0.counter.1.notify_one();
                }
                let text = res.text()?;
                let json = match serde_json::from_str::<Vec<RpcResponse<serde_json::Value>>>(&text) {
                    Ok(a) => a,
                    Err(_) => {
                        bail!("{:?}", serde_json::from_str::<RpcResponse<serde_json::Value>>(&text)?.error)
                    }
                };
                let res_res: Result<HashMap<req_id, serde_json::Value>, Error> = json.into_iter().map(|reply| {
                    Ok(match reply.result {
                        Some(b) => (reply.id.unwrap_or_else(req_id::new_v4), b),
                        _ => bail!("{:?}", reply.error),
                    })
                }).collect();
                let res = res_res?;
                self.inner().reqs = Vec::new();
                self.inner().resps.extend(res);
                Ok(())
            }

            fn send<T: for<'de> Deserialize<'de>>(&mut self) -> Result<HashMap<req_id, T>, Error> {
                self.flush()?;
                let res: Result<HashMap<req_id, T>, Error> = self.inner().resps.clone().into_iter().map(|(key, val)| Ok((key, serde_json::from_str(&serde_json::to_string(&val)?)?))).collect();
                if res.is_ok() {
                    self.inner().resps = HashMap::new();
                }
                res
            }
        }

        $(#[$struct_attr])*
        pub struct $struct_name {
            uri: String,
            user: Option<String>,
            pass: Option<String>,
            max_concurrency: usize,
            rps: usize,
            counter: (Mutex<usize>, Condvar),
            last_req: Mutex<std::time::Instant>,
            max_batch_size: usize,
        }

        impl $struct_name {
            pub fn new(uri: String, user: Option<String>, pass: Option<String>, max_concurrency: usize, rps: usize, max_batch_size: usize) -> Arc<Self> {
                Arc::new($struct_name {
                    uri,
                    user,
                    pass,
                    max_concurrency,
                    rps,
                    counter: (Mutex::new(0), Condvar::new()),
                    last_req: Mutex::new(std::time::Instant::now()),
                    max_batch_size,
                })
            }

            pub fn batcher<'a>(&'a self) -> (&'a Self, ReqBatcher<$struct_name>) {
                (self, ReqBatcher {
                    reqs: Vec::new(),
                    resps: HashMap::new(),
                    max_batch_size: self.max_batch_size,
                    phantom: PhantomData,
                })
            }

            $(
                $(
                    $(#[$attr_a])*
                    pub fn $method_a(&self$(, $arg_name_a: $arg_ty_a)*) -> Result<$return_ty_a, Error> {
                        let mut builder = rq::Client::new()
                            .post(&self.uri)
                            .header(rq::header::CONNECTION, rq::header::HeaderValue::from_static("close"));
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
                        if self.rps > 0 {
                            let wait = std::time::Duration::from_secs(1) / self.rps as u32;
                            let mut lock = self.last_req.lock().unwrap();
                            let elapsed = lock.elapsed();
                            if elapsed < wait {
                                std::thread::sleep(wait - elapsed);
                            }
                            *lock = std::time::Instant::now();
                            drop(lock);
                        }
                        if self.max_concurrency > 0 {
                            let mut lock = self.counter.0.lock().unwrap();
                            while *lock == self.max_concurrency {
                                lock = self.counter.1.wait(lock).unwrap();
                            }
                            if *lock > self.max_concurrency {
                                unreachable!();
                            }
                            *lock = *lock + 1;
                            drop(lock);
                        }
                        #[cfg(feature = "logging")] println!(stringify!($method_a));
                        let mut res = match builder.send() {
                            Ok(a) => a,
                            Err(e) => {
                                if self.max_concurrency > 0 {
                                    let mut lock = self.counter.0.lock().unwrap();
                                    *lock = *lock - 1;
                                    drop(lock);
                                    self.counter.1.notify_one();
                                }
                                Err(e)?
                            }
                        };
                        if self.max_concurrency > 0 {
                            let mut lock = self.counter.0.lock().unwrap();
                            *lock = *lock - 1;
                            drop(lock);
                            self.counter.1.notify_one();
                        }
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
                        let mut builder = rq::Client::new()
                            .post(&self.uri)
                            .header(rq::header::CONNECTION, rq::header::HeaderValue::from_static("close"));
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
                        if self.rps > 0 {
                            let wait = std::time::Duration::from_secs(1) / self.rps as u32;
                            let mut lock = self.last_req.lock().unwrap();
                            let elapsed = lock.elapsed();
                            if elapsed < wait {
                                std::thread::sleep(wait - elapsed);
                            }
                            *lock = std::time::Instant::now();
                            drop(lock);
                        }
                        if self.max_concurrency > 0 {
                            let mut lock = self.counter.0.lock().unwrap();
                            while *lock == self.max_concurrency {
                                lock = self.counter.1.wait(lock).unwrap();
                            }
                            if *lock > self.max_concurrency {
                                unreachable!();
                            }
                            *lock = *lock + 1;
                            drop(lock);
                        }
                        #[cfg(feature = "logging")] println!(stringify!($method_b));
                        let mut res = match builder.send() {
                            Ok(a) => a,
                            Err(e) => {
                                if self.max_concurrency > 0 {
                                    let mut lock = self.counter.0.lock().unwrap();
                                    *lock = *lock - 1;
                                    drop(lock);
                                    self.counter.1.notify_one();
                                }
                                Err(e)?
                            }
                        };
                        if self.max_concurrency > 0 {
                            let mut lock = self.counter.0.lock().unwrap();
                            *lock = *lock - 1;
                            drop(lock);
                            self.counter.1.notify_one();
                        }
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
        }
    };
}