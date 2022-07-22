#!/bin/sh
set -e

git pull

cargo build --release --bin=blog
cd migration

cargo build --release --bin=migration
cd ..

ssh "root@$1" "systemctl stop blog.service"

scp ./target/release/blog "root@$1:~/projects/b5/target/release/blog"
scp ./migration/target/release/migration "root@$1:~/projects/b5/migrate"

ssh "root@$1" "cd ~/projects/b5; git pull; ./migrate; systemctl enable --now blog.service"