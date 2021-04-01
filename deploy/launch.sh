MOCKNET_SERVER_DIR=/home/djp/Rust/mocknet-rust/target/debug
INDRADB_SERVER_DIR=/home/djp/Rust/indradb/target/debug

IP=172.23.66.208
MOCKNETPORT=3030
INDRADBPORT=3031
IMAGE=ubuntu:20.04

# launch indradb
sudo docker run \
--net=host \
--privileged \
-v /tmp:/tmp \
-v $INDRADB_SERVER_DIR:/workspace \
--name indradb-server \
-it \
-d $IMAGE \
/workspace/indradb-server -a $IP:$INDRADBPORT \
rocksdb /tmp/mocknet

# launch mocknet
sudo docker run \
--net=host \
--privileged \
-v $MOCKNET_SERVER_DIR:/workspace \
--name mocknet-server \
-it \
-d $IMAGE \
/workspace/mocknet_server  --warp-addr $IP:$MOCKNETPORT --indradb-addr $IP:$INDRADBPORT \
--cluster-config /workspace/cluster_config_template.json