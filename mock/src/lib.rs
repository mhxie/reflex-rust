#![warn(rust_2018_idioms)]

use rand::Rng;
use std::error::Error;
use std::io;
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
// use tokio::net::TcpStream;
use tokio::net::TcpSocket;
use tokio::sync::mpsc;

use stat::{average, percentile, Count, Perf};

pub async fn echo_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 4096];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                let resp = match n {
                    0 => return,
                    24 => 1048,
                    1048 => 24,
                    _ => n,
                };

                socket
                    .write_all(&buf[0..resp])
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}

pub async fn hello_ec2(addr: &str) -> Result<bool, Box<dyn Error>> {
    // simple hello world function to test VPC connectivity
    println!("connecting server @{}", addr);
    let mut stream = TcpStream::connect(&addr).await?;
    // let mut stream = TcpStream::connect_timeout(addr, Duration::from_secs(1)).await?;
    println!("created stream");

    let result = stream.write(b"hello world\n").await;
    println!("wrote to stream; success={:?}", result.is_ok());
    Ok(result.is_ok())
}

pub async fn pressure_ec2(
    address: &str,
    duration: u64,
    conns: u32,
    length: usize,
    rw_ratio: u32,
) -> Result<Perf, Box<dyn Error>> {
    // pressure a single server to get the peak performance
    println!("pressuring server@{}", address);
    let totltime = Duration::from_secs(duration);
    let (tx, mut rx) = mpsc::channel(32);
    let (ltx, mut lrx) = mpsc::channel(4096);
    let addr = address.parse().expect("Unable to part server address");

    for i in 0..conns {
        // created for each tasks
        let tx = tx.clone();
        let ltx = ltx.clone();

        tokio::spawn(async move {
            let mut sum = Count::default();
            let out_buf: Vec<u8> = vec![0; 4096];
            let mut in_buf: Vec<u8> = vec![0; 4096];
            let mut latency: Vec<Duration> = Vec::new();

            // Open a TCP stream to the socket address.
            let socket = TcpSocket::new_v4().unwrap();
            let mut stream = socket.connect(addr).await.unwrap();

            let start = Instant::now();
            let mut to_send = length;
            loop {
                let rtt;
                let last_t = Instant::now();
                if rand::thread_rng().gen_range(0..100) > rw_ratio {
                    to_send = 24;
                }
                match stream.write_all(&out_buf[0..to_send]).await {
                    Ok(_) => {
                        sum.send += 1;
                        sum.send_bytes += to_send as u64;
                    }
                    Err(_) => {
                        println!("Write error!");
                        break;
                    }
                };

                match stream.read(&mut in_buf).await {
                    Ok(n) => {
                        sum.recv += 1;
                        sum.recv_bytes += n as u64;
                        rtt = last_t.elapsed();
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(_) => {
                        println!("Read error!");
                        break;
                    }
                };
                latency.push(rtt);
                let elapsed = start.elapsed();
                if elapsed > totltime {
                    // println!("Done benchamarking for task-{}", i);
                    break;
                }
            }
            // send packet data back
            tx.send(sum).await.expect("Send network statistics failed.");
            match ltx.send(latency).await {
                Ok(_) => true,
                Err(_) => {
                    println!("Send latency metrics from {}-task failed", i);
                    false
                }
            }
            // Ok(())
        });
    }

    let mut sum = Count::default();
    let mut latency = Vec::new();

    for _ in 0..conns {
        let c = match rx.recv().await {
            Some(c) => c,
            None => Count::default(),
        };
        sum.recv += c.recv;
        sum.recv_bytes += c.recv_bytes;
        sum.send += c.send;
        sum.send_bytes += c.send_bytes;

        let lat = lrx.recv().await.unwrap();
        latency.extend(lat);
    }

    let qps = sum.send / duration;
    println!();

    let pts = vec![10, 50, 95, 99];
    let mut lats = vec![0; 4];

    latency.sort();
    for (i, pt) in (&pts).iter().enumerate() {
        lats[i] = percentile(*pt, &latency).as_micros() as u32;
        print!("| {}th-{:?} |", pt, lats[i]);
    }
    println!();
    let avg = average(&mut latency);
    println!("Average latency: {:?}", avg);
    println!();
    println!(
        "Send: {} bytes / {} Mbps",
        sum.send_bytes,
        sum.send_bytes / duration / 1024 / 128
    );
    println!(
        "Recv: {} bytes / {} Mbps",
        sum.recv_bytes,
        sum.recv_bytes / duration / 1024 / 128
    );
    let perf = Perf {
        iops: qps,
        p10: lats[0],
        p50: lats[1],
        p95: lats[2],
        p99: lats[3],
    };
    Ok(perf)
}

pub async fn pressure_multi_ec2(
    addresses: &[String],
    duration: u64,
    conns: u32,
    length: usize,
    rw_ratio: u32,
) -> Result<Perf, Box<dyn Error>> {
    // split data stream to multiple servers
    let totltime = Duration::from_secs(duration);
    let (tx, mut rx) = mpsc::channel(32);
    let (ltx, mut lrx) = mpsc::channel(4096);
    for address in addresses {
        let addr = address.parse().expect("Unable to part server address");
        println!("pressuring server @{}", address);
        for i in 0..conns {
            // created for each tasks
            let tx = tx.clone();
            let ltx = ltx.clone();

            tokio::spawn(async move {
                let mut sum = Count::default();
                let out_buf: Vec<u8> = vec![0; 8192];
                let mut in_buf: Vec<u8> = vec![0; 8192];
                let mut latency: Vec<Duration> = Vec::new();

                // Open a TCP stream to the socket address.
                let socket = TcpSocket::new_v4().unwrap();
                let mut stream = socket.connect(addr).await.unwrap();

                let start = Instant::now();
                let mut to_send = length;
                loop {
                    let rtt;
                    let last_t = Instant::now();
                    if rand::thread_rng().gen_range(0..100) > rw_ratio {
                        to_send = 24;
                    }
                    match stream.write_all(&out_buf[0..to_send]).await {
                        Ok(_) => {
                            sum.send += 1;
                            sum.send_bytes += to_send as u64;
                        }
                        Err(_) => {
                            println!("Write error!");
                            break;
                        }
                    };

                    match stream.read(&mut in_buf).await {
                        Ok(n) => {
                            sum.recv += 1;
                            sum.recv_bytes += n as u64;
                            rtt = last_t.elapsed();
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(_) => {
                            println!("Read error!");
                            break;
                        }
                    };
                    latency.push(rtt);
                    let elapsed = start.elapsed();
                    if elapsed > totltime {
                        break;
                    }
                }
                // send packet data back
                tx.send(sum).await.expect("Send network statistics failed.");
                match ltx.send(latency).await {
                    Ok(_) => true,
                    Err(_) => {
                        println!("Send latency metrics from {}-task failed", i);
                        false
                    }
                }
                // Ok(())
            });
        }
    }

    let mut sum = Count::default();
    let mut latency = Vec::new();
    for _ in addresses {
        for _ in 0..conns {
            let c = match rx.recv().await {
                Some(c) => c,
                None => Count::default(),
            };
            sum.recv += c.recv;
            sum.recv_bytes += c.recv_bytes;
            sum.send += c.send;
            sum.send_bytes += c.send_bytes;

            let lat = lrx.recv().await.unwrap();
            latency.extend(lat);
        }
    }

    let qps = sum.send / duration;
    let pts = vec![10, 50, 95, 99];
    let mut lats = vec![0; 4];

    latency.sort();
    for (i, pt) in (&pts).iter().enumerate() {
        lats[i] = percentile(*pt, &latency).as_micros() as u32;
    }

    let avg = average(&mut latency);
    println!("Average: {:?}", avg);
    let perf = Perf {
        iops: qps,
        p10: lats[0],
        p50: lats[1],
        p95: lats[2],
        p99: lats[3],
    };
    Ok(perf)
}
