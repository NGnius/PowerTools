//use usdpl_back::core::serdes::Primitive;
use usdpl_back::AsyncCallable;

pub struct AsyncIsh<
    In: Send + 'static,
    Out: Send + 'static,
    TS: (Fn(super::ApiParameterType) -> Result<In, String>) + Send + Sync,
    Gen: (Fn() -> SG) + Send + Sync,
    SG: (Fn(In) -> Out) + Send + Sync + 'static,
    TG: (Fn(Out) -> super::ApiParameterType) + Send + Sync,
> {
    pub trans_setter: TS, // assumed to be pretty fast
    pub set_get: Gen,     // probably has locks (i.e. slow)
    pub trans_getter: TG, // assumed to be pretty fast
}

#[async_trait::async_trait]
impl<
        In: Send + 'static,
        Out: Send + 'static,
        TS: (Fn(super::ApiParameterType) -> Result<In, String>) + Send + Sync,
        Gen: (Fn() -> SG) + Send + Sync,
        SG: (Fn(In) -> Out) + Send + Sync + 'static,
        TG: (Fn(Out) -> super::ApiParameterType) + Send + Sync,
    > AsyncCallable for AsyncIsh<In, Out, TS, Gen, SG, TG>
{
    async fn call(&self, params: super::ApiParameterType) -> super::ApiParameterType {
        let t_to_set = match (self.trans_setter)(params) {
            Ok(t) => t,
            Err(e) => return vec![e.into()],
        };
        let setter = (self.set_get)();
        let t_got = match tokio::task::spawn_blocking(move || setter(t_to_set)).await {
            Ok(t) => t,
            Err(e) => return vec![e.to_string().into()],
        };
        (self.trans_getter)(t_got)
    }
}

pub struct AsyncIshGetter<
    T: Send + 'static,
    Gen: (Fn() -> G) + Send + Sync,
    G: (Fn() -> T) + Send + Sync + 'static,
    TG: (Fn(T) -> super::ApiParameterType) + Send + Sync,
> {
    pub set_get: Gen,     // probably has locks (i.e. slow)
    pub trans_getter: TG, // assumed to be pretty fast
}

#[async_trait::async_trait]
impl<
        T: Send + 'static,
        Gen: (Fn() -> G) + Send + Sync,
        G: (Fn() -> T) + Send + Sync + 'static,
        TG: (Fn(T) -> super::ApiParameterType) + Send + Sync,
    > AsyncCallable for AsyncIshGetter<T, Gen, G, TG>
{
    async fn call(&self, _params: super::ApiParameterType) -> super::ApiParameterType {
        let getter = (self.set_get)();
        let t_got = match tokio::task::spawn_blocking(move || getter()).await {
            Ok(t) => t,
            Err(e) => return vec![e.to_string().into()],
        };
        (self.trans_getter)(t_got)
    }
}

pub struct Blocking<F: (Fn(super::ApiParameterType) -> super::ApiParameterType) + Send + Sync> {
    pub func: F,
}

#[async_trait::async_trait]
impl<F: (Fn(super::ApiParameterType) -> super::ApiParameterType) + Send + Sync> AsyncCallable
    for Blocking<F>
{
    async fn call(&self, params: super::ApiParameterType) -> super::ApiParameterType {
        (self.func)(params)
    }
}
