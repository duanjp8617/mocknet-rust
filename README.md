# mocknet-rust

Command for launching indradb container:
```shell
sudo docker run --net=host --privileged --entrypoint /workspace/indradb -v /tmp:/tmp -v /home/djp/indradb/target/debug:/workspace --name indradb -it -d ubuntu:18.04
```

Command for launching server_main container:
```shell
sudo docker run --net=host --privileged --entrypoint /workspace/server_main -v /home/djp/mocknet-rust/target/debug:/workspace --name mocknet -it -d ubuntu:18.04
```

# TODO
1. Change grpc formats. Add more information in the response. The k8s-server should perform a check to decide whether the devices contained in the requests are already in the process of creation.
2. If the creation fails and the state of the emunet is changed to error, the user should delete the emunet. During deletion, the emunet should be put into a garbage collector.
3. Add a interface for dumping all the failed emunets in the collector, and manually clean up the devices and links in the k8s nodes that are used to launch these failed emunets. 
4. Provide another interface to recycle the cleaned k8s nodes back into the ClusterInfo.