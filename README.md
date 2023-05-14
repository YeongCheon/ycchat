# Chat Server
## dev setting
### start redis
``` shell
sudo docker run --name redis -d -p 6379:6379 redis
```
### start surrealDB
``` shell
sudo docker run --rm -p 8000:8000 surrealdb/surrealdb:latest start --user root --pass root
```
### cargo run
``` shell
cargo run
```
