# mocknet-rust

Command for launching indradb container:
```shell
sudo docker run --net=host --privileged --entrypoint /workspace/indradb -v /tmp:/tmp -v /home/djp/indradb/target/debug:/workspace --name indradb -it -d ubuntu:18.04
```

Command for launching server_main container:
```shell
sudo docker run --net=host --privileged --entrypoint /workspace/server_main -v /home/djp/mocknet-rust/target/debug:/workspace --name mocknet -it -d ubuntu:18.04
```