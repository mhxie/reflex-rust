use netlify_lambda::{handler_fn, Context};
use serde_json::Value;

// use mock::hello_ec2;
use mock::pressure_ec2;
use stat::Args;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    netlify_lambda::run(handler_fn(handler)).await?;
    Ok(())
}

async fn handler(event: serde_json::Value, _: Context) -> Result<Value, Error> {
    let args: Args = serde_json::from_value(event).unwrap();
    println!("We got args: {:?}", args);
    // let res = match hello_ec2(args.addr.as_str()).await {
    //     Ok(true) => Perf::default(),
    //     Ok(false) => return Err("Unable to say hello".into()),
    //     Err(_) => return Err("Unexpected error".into()),
    // };
    let res = pressure_ec2(
        args.addr.as_str(),
        args.duration,
        args.number,
        args.length,
        args.rw_ratio,
    )
    .await
    .unwrap();
    println!("We got results: {:?}", res);
    Ok(serde_json::to_value(res).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock::echo_server;
    use serde_json::json;

    #[tokio::test]
    async fn handler_handles() {
        let addr = String::from("127.0.0.1:25000");

        tokio::spawn(async move {
            // run a echo server in the loop
            echo_server(&addr).await.unwrap()
        });

        let event = json!({
            "addr": "127.0.0.1:25000",
            "duration": 10,
            "number": 1,
            "length": 1024,
            "rw_ratio": 100,
        });
        let expected = json!({
            "iops": 0,
            "p10": 0,
            "p50": 0,
            "p95": 0,
            "p99": 0
        });
        tokio::spawn(async move {
            // test if we can get the results correctly
            let results = handler(event, Context::default()).await.unwrap();
            assert_eq!(results, expected);
        });
    }
}
